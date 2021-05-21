use natpmp::*;

#[tokio_main]
fn main() -> Result<()> {
    let n = new_tokio_natpmp().await?;
    n.send_port_mapping_request(Protocol::UDP, 4020, 4020, 30)
        .await?;
    match n.read_response_or_retry().await? {
        Response::UDP(ur) => {
            assert_eq!(ur.private_port(), 4020);
            assert_eq!(ur.public_port(), 4020); // Could be another port chosen by gateway
        }
        _ => {
            panic!("Expecting a udp response");
        }
    }
    Ok(())
}
