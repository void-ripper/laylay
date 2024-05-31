use std::path::PathBuf;

use clap::Parser;
use client::Client;
use errors::ServerErrors;
use server::ServerContext;
use tokio::net::TcpListener;

mod client;
mod database;
mod errors;
mod server;

#[derive(Parser)]
#[command(author, version)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:33033")]
    listen: String,
    #[arg(long, default_value = "info")]
    log: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt().with_env_filter(&args.log).init();

    let ret: Result<(), ServerErrors> = async {
        tracing::info!("-- start --");

        let data = PathBuf::from("data");

        if !data.exists() {
            std::fs::create_dir_all(&data)?;
        }

        let ctx = ServerContext::new(data.clone())?;
        let server = TcpListener::bind(&args.listen).await?;

        while let Ok((stream, _addr)) = server.accept().await {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                if let Err(e) = Client::new(ctx, stream).await {
                    tracing::error!("{e}");
                }
            });
        }

        tracing::info!("-- end --");

        Ok(())
    }
    .await;

    if let Err(e) = ret {
        tracing::error!("{e}");
    }
}
