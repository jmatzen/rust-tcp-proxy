use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "tcp_proxy")]
struct Opt {
    /// Bind address for the proxy
    #[structopt(long, default_value = "127.0.0.1:8080")]
    src: String,

    /// Destination address to forward traffic to
    #[structopt(long)]
    dst: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let listener = TcpListener::bind(&opt.src).await?;

    loop {
        let (inbound, _) = listener.accept().await?;
        let target_addr = opt.dst.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(inbound, &target_addr).await {
                eprintln!("Failed to handle client: {}", e);
            }
        });
    }
}

async fn handle_client(mut inbound: TcpStream, target_addr: &str) -> io::Result<()> {
    let mut outbound = TcpStream::connect(target_addr).await?;
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
