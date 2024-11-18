use wasmtime::{component::Component, Config, Engine};

#[test]
fn tcp_echo_server() {
    let wasm = std::fs::read(test_program_suite::TCP_ECHO_SERVER).unwrap();
    let config = Config::default();
    let engine = Engine::new(&config).unwrap();
    let _component = Component::new(&engine, wasm).unwrap();
}
