//! rustaria – A Rust download manager powered by aria2
//!
//! Entry point: bootstraps the application, parses CLI args, and starts
//! either the TUI or headless daemon mode.

mod aria2;
mod cli;
mod config;
mod db;
mod integration;
mod postprocess;
mod queue;
mod scheduler;
mod ui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::cli::Args;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("ferrum_dl=info".parse()?))
        .init();

    let args = Args::parse();

    // Load configuration
    let config = config::load(&args.config)?;

    // Initialize database
    let db = db::init(&config).await?;

    // Start aria2 daemon if configured
    let aria2 = aria2::Aria2::new(&config).await?;

    // Initialize job queue
    let queue = queue::JobQueue::new(db.clone(), aria2.clone());

    // Initialize scheduler
    let scheduler = scheduler::Scheduler::new(&config, queue.clone())?;

    if args.daemon {
        // Headless daemon mode
        tracing::info!("Starting rustaria in daemon mode...");
        run_daemon(config, queue, scheduler, aria2).await
    } else {
        // TUI mode
        tracing::info!("Starting rustaria TUI...");
        ui::tui::run(config, queue, scheduler, aria2).await
    }
}

async fn run_daemon(
    config: config::Config,
    queue: queue::JobQueue,
    scheduler: scheduler::Scheduler,
    aria2: aria2::Aria2,
) -> Result<()> {
    // Start integration services
    let mut handles = vec![];

    // Browser native messaging host
    if config.integration.native_messaging {
        handles.push(tokio::spawn(integration::native_host::run()));
    }

    // Clipboard monitor
    if config.general.clipboard_monitor {
        handles.push(tokio::spawn(integration::clipboard::run(
            config.general.clipboard_patterns.clone(),
            queue.clone(),
        )));
    }

    // aria2 WebSocket event listener
    handles.push(tokio::spawn(aria2.listen_events(queue.clone())));

    // Scheduler
    handles.push(tokio::spawn(scheduler.run()));

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down...");

    Ok(())
}
