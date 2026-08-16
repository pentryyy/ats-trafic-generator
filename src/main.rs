mod config;
mod dto;
mod generator;
mod services;

use crate::config::config::AppConfig;

use crate::generator::generator::run;
use anyhow::Result;

fn main() -> Result<()> {
    let cfg = AppConfig::load()?;

    if let Err(_) = run(&cfg) {
        std::process::exit(1);
    }

    Ok(())
}
