mod browser;
mod ai;
mod ui;

use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("Starting SyncFlo Browser...");
    
    // Initialize and run browser (must run on main thread on macOS)
    browser::Browser::new()?.run()?;
    
    Ok(())
}
