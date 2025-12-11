use crate::common::loader::LoggingConfig;
use std::fs::OpenOptions;
use std::str::FromStr;
use tracing_subscriber::{filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger(cfg: &LoggingConfig) {
    let logging_disabled = !cfg.stdout && cfg.file_path.trim().is_empty();
    if logging_disabled {
        return;
    }
    let level = LevelFilter::from_str(&cfg.level).unwrap_or(LevelFilter::INFO);
    let registry = tracing_subscriber::registry();
    if cfg.stdout {
        let layer = fmt::layer()
            .with_ansi(true)
            .with_target(false)
            .with_thread_names(true)
            .with_thread_ids(true);
        let subscriber = registry.with(level).with(layer);
        subscriber.init();
        return;
    }
    if !cfg.file_path.trim().is_empty() {
        if let Ok(file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cfg.file_path)
        {
            let layer = fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .with_writer(file);
            let subscriber = registry.with(level).with(layer);
            subscriber.init();
            return;
        }
    }
    let layer = fmt::layer();
    let subscriber = registry.with(level).with(layer);
    subscriber.init();
}
