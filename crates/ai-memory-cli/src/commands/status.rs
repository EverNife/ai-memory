//! `ai-memory status` — report runtime config and counts.
//!
//! M0 ships a placeholder that prints paths and version. M1 wires in the
//! store and replaces the placeholder with real counts.

use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::cli::StatusArgs;
use crate::config::Config;

#[derive(Debug, Serialize)]
struct Report<'a> {
    version: &'a str,
    data_dir: &'a Path,
    bind: &'a str,
    notes: &'static str,
}

/// Run the `status` subcommand.
///
/// # Errors
/// Returns an error if JSON serialization fails.
pub fn run(config: &Config, args: StatusArgs) -> Result<()> {
    let report = Report {
        version: env!("CARGO_PKG_VERSION"),
        data_dir: &config.data_dir,
        bind: &config.bind,
        notes: "M0 placeholder; counts arrive in M1",
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("ai-memory {}", report.version);
        println!("  data-dir: {}", report.data_dir.display());
        println!("  bind:     {}", report.bind);
        println!("  note:     {}", report.notes);
    }
    Ok(())
}
