use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use laylay_common::Message;
use tokio::{runtime::Runtime, sync::mpsc::Sender};
use tracing::{field::{Field, Visit}, span, Subscriber};

pub struct Logger {
    span_id_pool: AtomicU64,
    runtime: Arc<Runtime>,
    txch: Sender<Message>,
}

struct FieldCollect {
    data: Vec<String>,
}

impl Visit for FieldCollect {
    fn record_debug(&mut self, _field: &Field, value: &dyn std::fmt::Debug) {
        self.data.push(format!("{:?}", value));
    }
}

impl Logger {
    pub fn new(runtime: Arc<Runtime>, txch: Sender<Message>) -> Self {
        Self {
            runtime,
            txch,
            span_id_pool: AtomicU64::new(1),
        }
    }
}

impl Subscriber for Logger {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(self.span_id_pool.fetch_add(1, Ordering::SeqCst))
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let meta = event.metadata();
        let target = meta.target().to_string();
        if target.contains("polling") {
            return
        }

        let mut data = FieldCollect { data: Vec::new() };
        event.record(&mut data);

        let level = meta.level().as_str().to_string();
        let txch = self.txch.clone();
        self.runtime.spawn(async move {
            let ret = txch.send(Message::Log { msg: data.data.join(" "), level, target, }).await;
            if let Err(e) = ret {
                tracing::error!("{e}");
            }
        });
    }

    fn enter(&self, _span: &span::Id) {}

    fn exit(&self, _span: &span::Id) {}
}
