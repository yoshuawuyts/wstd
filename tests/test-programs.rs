use anyhow::{anyhow, Context, Result};
use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Config, Engine, Store,
};
use wasmtime_wasi::{pipe::MemoryOutputPipe, WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

struct Ctx {
    table: ResourceTable,
    wasi: WasiCtx,
    http: WasiHttpCtx,
}

impl WasiView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

fn run_in_wasmtime(wasm: &[u8], stdout: Option<MemoryOutputPipe>) -> Result<()> {
    let config = Config::default();
    let engine = Engine::new(&config).context("creating engine")?;
    let component = Component::new(&engine, wasm).context("loading component")?;

    let mut linker: Linker<Ctx> = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker).context("add wasi to linker")?;
    wasmtime_wasi_http::add_only_http_to_linker_sync(&mut linker)
        .context("add wasi-http to linker")?;

    let mut builder = WasiCtx::builder();
    builder.inherit_stderr().inherit_network();
    let wasi = match stdout {
        Some(stdout) => builder.stdout(stdout).build(),
        None => builder.inherit_stdout().build(),
    };
    let mut store = Store::new(
        &engine,
        Ctx {
            table: ResourceTable::new(),
            wasi,
            http: WasiHttpCtx::new(),
        },
    );

    let instance = linker.instantiate(&mut store, &component)?;
    let run_interface = instance
        .get_export(&mut store, None, "wasi:cli/run@0.2.0")
        .ok_or_else(|| anyhow!("wasi:cli/run missing?"))?;
    let run_func_export = instance
        .get_export(&mut store, Some(&run_interface), "run")
        .ok_or_else(|| anyhow!("run export missing?"))?;
    let run_func = instance
        .get_typed_func::<(), (Result<(), ()>,)>(&mut store, &run_func_export)
        .context("run as typed func")?;

    println!("entering wasm...");
    let (runtime_result,) = run_func.call(&mut store, ())?;
    runtime_result.map_err(|()| anyhow!("run returned an error"))?;
    println!("done");

    Ok(())
}

#[test]
fn tcp_echo_server() -> Result<()> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread::sleep;
    use std::time::Duration;

    println!("testing {}", test_programs_artifacts::TCP_ECHO_SERVER);
    let wasm = std::fs::read(test_programs_artifacts::TCP_ECHO_SERVER).context("read wasm")?;

    let pipe = wasmtime_wasi::pipe::MemoryOutputPipe::new(1024 * 1024);
    let write_end = pipe.clone();
    let wasmtime_thread = std::thread::spawn(move || run_in_wasmtime(&wasm, Some(write_end)));

    'wait: loop {
        sleep(Duration::from_millis(100));
        for line in pipe.contents().split(|c| *c == b'\n') {
            if line.starts_with(b"Listening on") {
                break 'wait;
            }
        }
    }

    let mut tcpstream =
        TcpStream::connect("127.0.0.1:8080").context("connect to wasm echo server")?;
    println!("connected to wasm echo server");

    const MESSAGE: &[u8] = b"hello, echoserver!\n";

    tcpstream.write_all(MESSAGE).context("write to socket")?;
    println!("wrote to echo server");

    let mut readback = Vec::new();
    tcpstream
        .read_to_end(&mut readback)
        .context("read from socket")?;

    println!("read from wasm server");
    assert_eq!(MESSAGE, readback);

    if wasmtime_thread.is_finished() {
        wasmtime_thread.join().expect("wasmtime panicked")?;
    }
    Ok(())
}

#[test]
fn http_get() -> Result<()> {
    println!("testing {}", test_programs_artifacts::HTTP_GET);
    let wasm = std::fs::read(test_programs_artifacts::HTTP_GET).context("read wasm")?;
    run_in_wasmtime(&wasm, None)
}

#[test]
fn http_first_byte_timeout() -> Result<()> {
    println!(
        "testing {}",
        test_programs_artifacts::HTTP_FIRST_BYTE_TIMEOUT
    );
    let wasm =
        std::fs::read(test_programs_artifacts::HTTP_FIRST_BYTE_TIMEOUT).context("read wasm")?;
    run_in_wasmtime(&wasm, None)
}
