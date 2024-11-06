use std::sync::Arc;

use natpmp::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let n = Arc::new(new_tokio_natpmp().await?);

    let n_cloned = n.clone();
    tokio::spawn(async {
        let n = n_cloned;
        loop {
            println!("Sending request...");
            if let Err(e) = n
                .send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)
                .await
            {
                eprintln!("Sending request err: {}", e);
                break;
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    loop {
        println!("Waiting response...");
        match n.read_response_or_retry().await? {
            Response::UDP(ur) => {
                assert_eq!(ur.private_port(), 4020);
                assert_eq!(ur.public_port(), 4020); // Could be another port chosen by gateway
            }
            _ => {
                eprintln!("Expecting a udp response");
                break;
            }
        }
    }

    Ok(())
}
