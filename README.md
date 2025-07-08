<h1 align="center">wstd</h1>
<div align="center">
  <strong>
    An async standard library for Wasm Components and WASI 0.2
  </strong>
</div>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/wstd">
    <img src="https://img.shields.io/crates/v/wstd.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/wstd">
    <img src="https://img.shields.io/crates/d/wstd.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/wstd">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/wstd">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/wstd/releases">
      Releases
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/wstd/blob/master.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>


This is a minimal async standard library written exclusively to support Wasm
Components. It exists primarily to enable people to write async-based
applications in Rust before async-std, smol, or tokio land support for Wasm
Components and WASI 0.2. Once those runtimes land support, it is recommended
users switch to use those instead.

## Examples

**TCP echo server**

```rust
use wstd::io;
use wstd::iter::AsyncIterator;
use wstd::net::TcpListener;
use wstd::runtime::block_on;

fn main() -> io::Result<()> {
    block_on(async move {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Listening on {}", listener.local_addr()?);
        println!("type `nc localhost 8080` to create a TCP client");

        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            println!("Accepted from: {}", stream.peer_addr()?);
            io::copy(&stream, &stream).await?;
        }
        Ok(())
    })
}
```

## Installation
```sh
$ cargo add wstd
```

## Safety
This crate uses ``#![forbid(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/yoshuawuyts/wstd/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/yoshuawuyts/wstd/labels/good%20first%20issue
[help-wanted]: https://github.com/yoshuawuyts/wstd/labels/help%20wanted

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
