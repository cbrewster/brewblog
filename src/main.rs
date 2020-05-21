mod build;
mod config;
mod markdown;
mod page;

use crate::build::build;
use anyhow::{Context, Result};
use clap::Clap;
use notify::{watcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Clap, Debug)]
#[clap(version = "0.1.3", author = "Connor Brewster")]
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

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.command {
        Command::New(new_opts) => new_site(new_opts)?,
        Command::Build => build()?,
        Command::Serve => serve().await?,
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

// Simple static file server
async fn serve() -> Result<()> {
    println!("Buildig...");
    build()?;
    println!("Serving blog at 127.0.0.1:3030");

    let (sender, receiver) = channel();
    let mut watcher = watcher(sender, Duration::from_secs(2)).unwrap();
    // TODO: Need to read from config...
    watcher.watch("content", RecursiveMode::Recursive).unwrap();
    watcher
        .watch("templates", RecursiveMode::Recursive)
        .unwrap();
    std::thread::spawn(move || loop {
        match receiver.recv() {
            Ok(_) => {
                println!("Building...");
                if let Err(error) = build() {
                    println!("{}", error);
                };
            }
            Err(e) => println!("watch error: {}", e),
        }
    });

    let static_files = warp::fs::dir("public");

    warp::serve(static_files).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
