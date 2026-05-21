//! Structured tracing setup.
//!
//! `RUST_LOG` honoured first; otherwise we fall back to the configured
//! [`Config::log_level`]. The appender's own module is forced to `warn` to
//! avoid the feedback loop that filled 137 GB of disk for agentmemory #519.
//!
//! [`Config::log_level`]: crate::config::Config::log_level

use std::fs;

use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::Config;

/// Initialise the global tracing subscriber.
///
/// Returns a guard whose drop flushes any pending log lines. Keep the guard
/// alive for the duration of `main()`.
///
/// # Errors
/// Returns an error if the log directory cannot be created.
pub fn init(config: &Config) -> Result<WorkerGuard> {
    let log_dir = config.data_dir.join("logs");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("creating log directory {}", log_dir.display()))?;

    let appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "ai-memory.log");
    let (file_writer, guard) = tracing_appender::non_blocking(appender);

    let default_filter = format!("{},tracing_appender=warn", config.log_level);
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_writer(std::io::stderr);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(file_writer);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
