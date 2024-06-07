use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Weak,
    },
};

use tokio::sync::RwLock;

pub type NodePtr = Arc<Node>;
static mut NODE_ID_POOL: AtomicU64 = AtomicU64::new(1);

pub struct Node {
    id: u64,
    me: Weak<Self>,
    parent: RwLock<Option<NodePtr>>,
    children: RwLock<HashMap<u64, NodePtr>>,
}

impl Node {
    pub fn new() -> NodePtr {
        let id = unsafe { NODE_ID_POOL.fetch_add(1, Ordering::SeqCst) };
        Arc::new_cyclic(|n| Self {
            id,
            me: n.clone(),
            parent: RwLock::new(None),
            children: RwLock::new(HashMap::new()),
        })
    }

    pub async fn add_child(&self, ch: NodePtr) {
        if let Some(p) = ch.parent.read().await.clone() {
            p.remove_child(ch.clone());
        }

        *ch.parent.write().await = self.me.upgrade();
        self.children.write().await.insert(ch.id, ch);
    }

    pub async fn remove_child(&self, ch: NodePtr) {
        self.children.write().await.remove(&ch.id);
        ch.parent.write().await.take();
    }
}
