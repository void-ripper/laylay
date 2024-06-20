use laylay_common::{Info, Message, SecretKey, Version};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{errors::ClientError, logger::Logger};

pub struct Network {}

impl Network {
    pub async fn connect(prikey: &SecretKey) -> Result<Self, ClientError> {
        let addr = if cfg!(target_os = "android") {
            "192.168.1.9"
        } else {
            "127.0.0.1"
        };

        let public = prikey.public_key().to_sec1_bytes();
        let mut stream = TcpStream::connect((addr, 33033)).await?;
        let greeting = Message::Greeting {
            pubkey: public.into(),
            version: Version::get(),
            info: Info::new()?,
        };
        laylay_common::write_greeting(&mut stream, &greeting).await?;

        let ret = laylay_common::read_greeting(&mut stream).await?;
        if let Message::Greeting {
            pubkey,
            version: _,
            info: _,
        } = ret
        {
            let shared = laylay_common::shared_secret(pubkey, &prikey);
            let (mut rx, mut tx) = stream.into_split();
            let (txch, mut rxch) = mpsc::channel::<Message>(10);

            tracing::subscriber::set_global_default(Logger::new(runtime, txch))?;

            let shared0 = shared.clone();
            tokio::spawn(async move {
                while let Some(msg) = rxch.recv().await {
                    if let Err(e) = laylay_common::write(&shared0, &mut tx, &msg).await {
                        tracing::error!("{e}");
                    }
                }
            });

            tokio::spawn(async move {
                loop {
                    let ret = laylay_common::read(&shared, &mut rx).await;
                    match ret {
                        Ok(msg) => {}
                        Err(e) => {
                            tracing::error!("{e}");
                        }
                    }
                }
            });
        }
        
        Ok(Self {})
    }
}
