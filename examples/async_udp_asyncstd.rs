use std::sync::Arc;

use natpmp::*;
use std::time::Duration;

fn main() -> Result<()> {
    use async_std::future;
    use async_std::task;

    task::block_on(async {
        let n = Arc::new(new_async_std_natpmp().await.unwrap());

        let n_cloned = n.clone();
        task::spawn(async {
            let n = n_cloned;
            loop {
                println!("Sending request...");
                if let Err(e) = n
                    .send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)
                    .await
                {
                    eprintln!("Sending request error: {}", e);
                    break;
                }
                let _ = future::timeout(Duration::from_secs(3), async {}).await;
            }
        });

        loop {
            match n.read_response_or_retry().await {
                Ok(Response::UDP(ur)) => {
                    assert_eq!(ur.private_port(), 4020);
                    assert_eq!(ur.public_port(), 4020); // Could be another port chosen by gateway
                }
                _ => {
                    eprintln!("Expecting a udp response");
                    break;
                }
            }
        }
    });

    Ok(())
}
