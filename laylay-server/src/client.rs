
use std::sync::Arc;

use laylay_common::{read_greeting, shared_secret, write_greeting, Bytes, Message, Version};
use tokio::{net::TcpStream, sync::mpsc::{channel, Sender}};

use crate::{errors::ServerErrors, server::ServerContext};

pub struct Client {
    server: Arc<ServerContext>,
    pubkey: Bytes,
    version: Version,
    txch: Sender<Message>,
}

impl Client {
    pub async fn new(ctx: Arc<ServerContext>, mut stream: TcpStream) -> Result<Arc<Self>, ServerErrors> {
        write_greeting(&mut stream, &ctx.greeting).await?;
        
        let msg = read_greeting(&mut stream).await?;

        if let Message::Greeting { pubkey, version } = msg {
            let (mut rx, mut tx) = stream.into_split();
            let (txch, mut rxch) = channel(10);
            let client = Arc::new(Self { server: ctx.clone(), pubkey: pubkey.clone(), version, txch });
            let shared = shared_secret(pubkey.clone(), &ctx.prikey);

            let shared0 = shared.clone();
            let cl0 = client.clone();
            tokio::spawn(async move {
                loop {
                    let ret = laylay_common::read(&shared0, &mut rx)
                        .await.map_err(|e| ServerErrors::Internal(e.to_string()));

                    match ret {
                        Ok(msg) => {
                            if let Err(e) = cl0.handle_message(msg).await {
                                tracing::error!("{e}");
                            }
                        }
                        Err(e) => {
                            tracing::error!("{e}");
                            break;
                        }
                    }
                }
            });

            tokio::spawn(async move {
                while let Some(msg) = rxch.recv().await {
                    let ret = laylay_common::write(&shared, &mut tx, &msg).await;

                    if let Err(e) = ret {
                        tracing::error!("{e}");
                    }
                }
            });

            ctx.add_client(client.pubkey.clone(), client.clone()).await;

            Ok(client)
        } else {
            Err(ServerErrors::Internal(
                "client did not send greeting".to_owned(),
            ))
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), ServerErrors> {
        match msg {
            Message::Log { msg, target, level } => {
                tracing::info!("{level} {target}: {msg}");
                self.server.db.save_log(&level, &target, &msg).await?;
            }
            _ => {}
        }

        Ok(())
    }
}