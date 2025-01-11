use anyhow::Result;

fn run_in_wasmtime(wasm: &str) -> Result<()> {
    use clap::Parser;
    use wasmtime_cli::commands::ServeCommand;

    // Run wasmtime serve.
    // Enable -Scli because we build with the default adapter rather than the
    // proxy adapter.
    // Disable logging so that Wasmtime's tracing_subscriber registration
    // doesn't conflict with the test harness' registration.
    let serve =
        match ServeCommand::try_parse_from(["serve", "-Scli", "-Dlogging=n", wasm].into_iter()) {
            Ok(serve) => serve,
            Err(e) => {
                dbg!(&e);
                return Err(e.into());
            }
        };

    serve.execute()
}

#[test_log::test]
fn http_server() -> Result<()> {
    use std::net::TcpStream;
    use std::thread::sleep;
    use std::time::Duration;

    // Start a `wasmtime serve` server.
    let wasmtime_thread =
        std::thread::spawn(move || run_in_wasmtime(test_programs_artifacts::HTTP_SERVER));

    // Clumsily wait for the server to accept connections.
    'wait: loop {
        sleep(Duration::from_millis(100));
        if TcpStream::connect("127.0.0.1:8080").is_ok() {
            break 'wait;
        }
    }

    // Do some tests!

    let body: String = ureq::get("http://127.0.0.1:8080").call()?.into_string()?;
    assert_eq!(body, "Hello, wasi:http/proxy world!\n");

    match ureq::get("http://127.0.0.1:8080/fail").call() {
        Ok(body) => {
            unreachable!("unexpected success from /fail: {:?}", body);
        }
        Err(ureq::Error::Transport(_transport)) => {}
        Err(other) => {
            unreachable!("unexpected error: {:?}", other);
        }
    }

    const MESSAGE: &[u8] = b"hello, echoserver!\n";

    let body: String = ureq::get("http://127.0.0.1:8080/echo")
        .send(MESSAGE)?
        .into_string()?;
    assert_eq!(body.as_bytes(), MESSAGE);

    let test_headers = [
        ("Red", "Rhubarb"),
        ("Orange", "Carrots"),
        ("Yellow", "Bananas"),
        ("Green", "Broccoli"),
        ("Blue", "Blueberries"),
        ("Purple", "Beets"),
    ];

    let mut response = ureq::get("http://127.0.0.1:8080/echo-headers");
    for (name, value) in test_headers {
        response = response.set(name, value);
    }
    let response = response.call()?;

    assert!(response.headers_names().len() >= test_headers.len());
    for (name, value) in test_headers {
        assert_eq!(response.header(name), Some(value));
    }

    if wasmtime_thread.is_finished() {
        wasmtime_thread.join().expect("wasmtime panicked")?;
    }

    Ok(())
}
