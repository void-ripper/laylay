use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Weak,
    },
};

use tokio::sync::{Mutex, RwLock};

use crate::math::matrix::{self, Matrix};

use super::{
    drawable::DrawablePtr,
    material::{Material, DEFAULT},
};

pub type NodePtr = Arc<Node>;
static NODE_ID_POOL: AtomicU32 = AtomicU32::new(1);

pub struct Node {
    id: u32,
    me: Weak<Self>,
    pub transform: RwLock<Matrix>,
    pub world_transform: RwLock<Matrix>,
    parent: RwLock<Option<NodePtr>>,
    children: RwLock<HashMap<u32, NodePtr>>,
    drawable: Mutex<Option<DrawablePtr>>,
    pub material: Mutex<Option<Material>>,
}

impl Node {
    pub fn new() -> NodePtr {
        let id = NODE_ID_POOL.fetch_add(1, Ordering::SeqCst);
        Arc::new_cyclic(|n| Self {
            id,
            me: n.clone(),
            transform: RwLock::new(matrix::new()),
            world_transform: RwLock::new(matrix::new()),
            parent: RwLock::new(None),
            children: RwLock::new(HashMap::new()),
            drawable: Mutex::new(None),
            material: Mutex::new(None),
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

    pub async fn set_drawable(&self, drw: DrawablePtr) {
        *self.drawable.lock().await = Some(drw.clone());
        *self.material.lock().await = Some(DEFAULT.clone());
        drw.instances
            .write()
            .await
            .insert(self.id, self.me.upgrade().unwrap());
    }

    pub fn update<'a>(&'a self, pwt: &'a [f32; 16]) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            {
                let mut wt = self.world_transform.write().await;
                matrix::identity(&mut wt);
                matrix::mul_assign(&mut wt, &pwt);
                matrix::mul_assign(&mut wt, &*self.transform.read().await);
            }

            let wt = self.world_transform.read().await;
            for ch in self.children.read().await.values() {
                ch.update(&*wt).await;
            }
        })
    }
}
