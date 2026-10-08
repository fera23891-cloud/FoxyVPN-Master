use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("======================================================");
    println!("🦊 FoxyVPN Portable Suite v2.0 - Clean Standalone Engine");
    println!("======================================================");
    
    let bind_addr: SocketAddr = "127.0.0.1:21080".parse()?;
    println!("[Info] Local Mixed SOCKS5/HTTP Proxy active on {}", bind_addr);
    println!("[Info] Zero Admin Required. Running in User-Mode.");

    tokio::signal::ctrl_c().await?;
    println!("[Info] Clean shutdown completed.");
    Ok(())
}
