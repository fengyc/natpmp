natpmp
======

[![Main](https://github.com/fengyc/natpmp/actions/workflows/main.yml/badge.svg)](https://github.com/fengyc/natpmp/actions/workflows/main.yml)

[![Nightly](https://github.com/fengyc/natpmp/actions/workflows/nightly.yml/badge.svg)](https://github.com/fengyc/natpmp/actions/workflows/nightly.yml)

NAT-PMP client library in rust, a rust implementation of the c library libnatpmp([https://github.com/miniupnp/libnatpmp](https://github.com/miniupnp/libnatpmp)).

*Note*: `src/getgateway.h` and `src/getgateway.c` are from [https://github.com/miniupnp/libnatpmp](https://github.com/miniupnp/libnatpmp) .

Versions
--------

Version 0.2.x supports rust 2018 edition.

Version 0.3.x supports tokio and async-std.

Example
-------

Create a natpmp object with system default gateway:

    use natpmp::*

    let n = Natpmp::new()?

Or a specified gataway:

    use std::str::FromStr;
    use natpmp::*;

    let n = Natpmp::new("192.168.0.1").parse.unwrap())?

To determine the external address, send a public address request:

    n.send_public_address_request()?;

To add a port mapping, send a port mapping request:

    n.send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)?;

And then read response after a few milliseconds:

    use std::thread;
    use std::time::Duration;

    thread::sleep(Duration::from_millis(250));
    let response = n.read_response_or_retry()?;

Check response type and and result:

    match response {
        Response::Gateway(gr) => {}
        Response::UDP(ur) => {}
        Response::TCP(tr) => {}
    }

Async
------

Enable feature `tokio` or `async-std` in Cargo.toml (default feature `tokio`).

    [dependencies]
    natpmp = { version = "0.3", features = ["tokio"] }

Or

    [dependencies]
    natpmp = { version = "0.3", features = ["async-std"] }

License
-------

MIT