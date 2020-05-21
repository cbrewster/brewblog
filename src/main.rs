mod build;
mod config;
mod page;

use crate::build::build;
use anyhow::{Context, Result};
use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Connor Brewster")]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap, Debug)]
enum Command {
    New(NewCommand),

    #[clap(about = "Builds the static site")]
    Build,

    #[clap(about = "Builds the static site and serves the site using a local server")]
    Serve,
}

#[derive(Clap, Debug)]
#[clap(about = "Create a new static site")]
struct NewCommand {
    #[clap(about = "The name of the site to be created")]
    name: String,

    #[clap(about = "The directory to create the site in (If blank, the site name will be used)")]
    directory: Option<String>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.command {
        Command::New(new_opts) => new_site(new_opts)?,
        Command::Build => build()?,
        Command::Serve => serve(),
    }
    Ok(())
}

fn new_site(opts: NewCommand) -> Result<()> {
    let path = PathBuf::from(opts.directory.as_ref().unwrap_or(&opts.name));

    std::fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create site directory: {:?}", path))?;

    println!("Creating new site {:?} at {:?}", opts.name, path);
    Ok(())
}

fn serve() {
    println!("Serving...");
}
