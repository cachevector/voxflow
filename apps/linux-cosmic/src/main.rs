//! VoxFlow Linux/COSMIC desktop app (Phase 2).
//! Uses shared Rust core with clipboard-based text insertion.

mod app;
mod insert;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("voxflow=info".parse()?))
        .init();

    app::run()
}
