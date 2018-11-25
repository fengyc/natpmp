libnatpmp
=========

[![Build Status](https://travis-ci.org/fengyc/libnatpmp.svg?branch=master)](https://travis-ci.org/fengyc/libnatpmp)

NAT-PMP client library in rust, a rust implementation of the c library libnatpmp([https://github.com/miniupnp/libnatpmp]).

*Note*: `src/getgateway.h` and `src/getgateway.c` are from [https://github.com/miniupnp/libnatpmp] .

Example
-------

Create a natpmp object with system default gateway:

    use libnatpmp::*

    let natpmp = Natpmp::new()?

Or a specified gataway:

    use std::str::FromStr;
    use libnatpmp::*;

    let natpmp = Natpmp::new("192.168.0.1").parse.unwrap())?

To determine the external address, send a public address request:

    natpmp.send_public_address_request()?;

To add a port mapping, send a port mapping request:

    natpmp.send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)?;

And then read response after a few milliseconds:

    use std::thread;
    use std::time::Duration;

    thread::sleep(Duration::from_millis(250));
    let response = natpmp.read_response_or_retry()?;

Check response type and and result:

    match response {
        Response::Gateway(gr) => {}
        Response::UDP(ur) => {}
        Response::TCP(tr) => {}
    }

License
-------

MIT