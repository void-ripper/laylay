use std::{collections::HashMap, path::PathBuf, sync::Arc};

use laylay_common::{get_private_key, Bytes, Info, Message, SecretKey, Version};
use tokio::sync::RwLock;

use crate::{client::Client, database::Database, errors::ServerErrors};

pub struct ServerContext {
    pub prikey: SecretKey,
    pub db: Database,
    pub greeting: Message,
    pub clients: RwLock<HashMap<Bytes, Arc<Client>>>,
}

impl ServerContext {
    pub fn new(folder: PathBuf) -> Result<Arc<Self>, ServerErrors> {
        let prikey = get_private_key(folder.clone())?;
        let greeting = Message::Greeting {
            pubkey: prikey.public_key().to_sec1_bytes().into(),
            version: Version::get(),
            info: Info::new(),
        };

        Ok(Arc::new(Self {
            prikey,
            db: Database::new(folder)?,
            greeting,
            clients: RwLock::new(HashMap::new()),
        }))
    }

    pub async fn add_client(&self, pubkey: Bytes, cl: Arc<Client>) {
        self.clients.write().await.insert(pubkey, cl);
    }
}
