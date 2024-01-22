use std::error::Error;

use clap::Parser;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(author, version)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:3333")]
    listen: String,
    #[arg(long, default_value = "info")]
    log: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt().with_env_filter(&args.log).init();

    let ret: Result<(), Box<dyn Error>> = async {
        tracing::info!("-- start --");

        let server = TcpListener::bind(&args.listen).await?;

        tracing::info!("-- end --");

        Ok(())
    }
    .await;

    if let Err(e) = ret {
        tracing::error!("{e}");
    }
}
