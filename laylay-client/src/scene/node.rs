use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Weak,
    },
};

use tokio::sync::RwLock;

use crate::math::matrix::{self, Matrix};

pub type NodePtr = Arc<Node>;
static NODE_ID_POOL: AtomicU64 = AtomicU64::new(1);

pub struct Node {
    id: u64,
    me: Weak<Self>,
    pub transform: RwLock<Matrix>,
    parent: RwLock<Option<NodePtr>>,
    children: RwLock<HashMap<u64, NodePtr>>,
}

impl Node {
    pub fn new() -> NodePtr {
        let id = NODE_ID_POOL.fetch_add(1, Ordering::SeqCst);
        Arc::new_cyclic(|n| Self {
            id,
            me: n.clone(),
            transform: RwLock::new(matrix::new()),
            parent: RwLock::new(None),
            children: RwLock::new(HashMap::new()),
        })
    }

    pub async fn add_child(&self, ch: NodePtr) {
        if let Some(p) = ch.parent.read().await.clone() {
            p.remove_child(ch.clone()).await;
        }

        *ch.parent.write().await = self.me.upgrade();
        self.children.write().await.insert(ch.id, ch);
    }

    pub async fn remove_child(&self, ch: NodePtr) {
        self.children.write().await.remove(&ch.id);
        ch.parent.write().await.take();
    }
}
