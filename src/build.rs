use crate::config::SiteConfig;
use anyhow::{Context, Result};

pub fn build() -> Result<()> {
    let config = std::fs::read_to_string("Config.toml").context("Could not open Config.toml")?;
    let config: SiteConfig = toml::from_str(&config).context("Failed to parse Config.toml")?;
    dbg!(&config);
    println!("Building...");
    Ok(())
}
