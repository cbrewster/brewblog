use crate::config::SiteConfig;
use crate::page::{Page, PageMetadata};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct BuildContext {
    pub output_dir: PathBuf,
    pub content_dir: PathBuf,
}

impl BuildContext {
    pub fn output_path(&self, content: impl AsRef<Path>) -> Result<PathBuf> {
        Ok(self
            .output_dir
            .join(content.as_ref().strip_prefix(&self.content_dir)?))
    }

    pub fn page_path(&self, page: impl AsRef<Path>) -> Result<PathBuf> {
        // If this is an index file, if so don't nest in a directory.
        if page.as_ref().file_name().and_then(|f| f.to_str()) == Some("index.md") {
            Ok(self.output_path(page.as_ref())?.with_extension("html"))
        } else {
            Ok(self
                .output_path(page.as_ref())?
                .with_extension("")
                .join("index")
                .with_extension("html"))
        }
    }
}

pub fn build() -> Result<()> {
    let config = std::fs::read_to_string("Config.toml").context("Could not open Config.toml")?;
    let config: SiteConfig = toml::from_str(&config).context("Failed to parse Config.toml")?;
    dbg!(&config);

    let output_dir = &config.output_dir;
    // Make sure output directory exists and is empty
    std::fs::remove_dir_all(output_dir)?;
    std::fs::create_dir_all(output_dir)?;

    let context = BuildContext {
        output_dir: PathBuf::from(&config.output_dir),
        content_dir: PathBuf::from(&config.content_dir),
    };

    build_directory(&config.content_dir, &context)?;

    println!("Building...");
    Ok(())
}

fn build_directory(dir: impl AsRef<Path>, context: &BuildContext) -> Result<()> {
    println!("Building directory {:?}", dir.as_ref());
    let out_dir = context.output_path(dir.as_ref())?;
    std::fs::create_dir_all(out_dir)?;

    let mut index = Vec::new();

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            build_directory(entry.path(), context)?;
        } else if metadata.is_file() {
            index.push(build_page(entry.path(), context)?);
        }
    }

    // Build index for pages...
    dbg!(&index);

    Ok(())
}

fn build_page(path: impl AsRef<Path>, context: &BuildContext) -> Result<PageMetadata> {
    let path = path.as_ref();

    println!("Building page: {:?}", path);

    let page = Page::parse(path, context)?;

    // Safe to unwrap since we always put stuff in a parent directory.
    std::fs::create_dir_all(&page.metadata.out_path.parent().unwrap()).with_context(|| {
        format!(
            "Failed to create page dir for page {:?}",
            page.metadata.out_path
        )
    })?;

    std::fs::write(&page.metadata.out_path, "<h1>YIPPEE</h1>")
        .with_context(|| format!("Failed to write page to {:?}", page.metadata.out_path))?;

    Ok(page.metadata)
}
