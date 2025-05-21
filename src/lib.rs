pub mod tracing_utils;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::SubscriberExt,util::SubscriberInitExt, EnvFilter, Layer, Registry};

pub fn setup_tracing() {
    // Create an EnvFilter that reads from RUST_LOG with INFO as default
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Configure logging based on environment
    if std::env::var("IS_SYSTEMD_SERVICE").is_ok() {
        // Use systemd formatting when running as a service
        let journald_layer = tracing_journald::layer().expect("Failed to create journald layer");
        tracing_subscriber::registry()
            .with(journald_layer)
            .with(env_filter)
            .init();
    } else {
        // Use standard formatting for non-systemd environments
        tracing_subscriber::fmt()
            .with_ansi(true)
            .with_target(true)
            .with_env_filter(env_filter)
            .init();
    }
}


#[derive(Serialize, Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

pub type LogCache = Arc<RwLock<Vec<LogEntry>>>;

#[derive(Deserialize)]
pub struct LogQuery {
    pub level: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

pub fn setup_tracing_with_broadcast(tx: broadcast::Sender<LogEntry>, cache: LogCache) {
    let layer = BroadcastLogLayer { tx, cache };
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(tracing_subscriber::fmt::layer().json())
        .with(layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

struct BroadcastLogLayer {
    tx: broadcast::Sender<LogEntry>,
    cache: LogCache,
}

impl<S: Subscriber> Layer<S> for BroadcastLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = TracingVisitor::default();
        event.record(&mut visitor);

        // 构建 Arc 包裹的日志对象
        let log = Arc::new(LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: event.metadata().level().to_string(),
            target: event.metadata().target().to_string(),
            message: visitor.message.unwrap_or_else(|| "<no message>".to_string()),
        });

        // 广播日志副本（需要 LogEntry 实现 Clone）
        let _ = self.tx.send((*log).clone());

        let cache = self.cache.clone();
        let log_clone = log.clone();

        // 异步缓存 + 持久化
        tokio::spawn(async move {
            {
                let mut logs = cache.write().await;
                logs.push((*log_clone).clone());
                if logs.len() > 1000 {
                    let len = logs.len();
                    logs.drain(0..(len - 1000));
                }
            }

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs.jsonl")
            {
                let _ = writeln!(file, "{}", serde_json::to_string(&*log_clone).unwrap());
            }
        });
    }
}

#[derive(Default)]
pub struct TracingVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for TracingVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }
}

