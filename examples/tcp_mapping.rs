extern crate natpmp;

use std::thread;
use std::time::Duration;
use natpmp::*;

fn main() -> Result<()> {
    let mut n = Natpmp::new()?;
    n.send_port_mapping_request(Protocol::TCP, 4020, 4020, 30)?;
    // sleep for a while
    thread::sleep(Duration::from_millis(100));
    match n.read_response_or_retry() {
        Err(e) => {
            match e {
                Error::NATPMP_TRYAGAIN => println!("Try again later"),
                _ => return Err(e)
            }
        }
        Ok(Response::TCP(tr)) => {
            assert_eq!(tr.private_port(), 4020);
            assert_eq!(tr.public_port(), 4020); // Could be another port chosen by gateway
        }
        _ => {
            panic!("Expecting a tcp response");
        }
    }
    Ok(())
}