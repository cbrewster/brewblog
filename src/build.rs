use crate::config::SiteConfig;
use crate::markdown::MarkdownRenderer;
use crate::page::{Page, PageMetadata};
use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tera::Tera;

pub struct BuildContext {
    pub output_dir: PathBuf,
    pub content_dir: PathBuf,
    pub templates: Tera,
    pub site_context: SiteContext,
    pub markdown_renderer: MarkdownRenderer,
}

#[derive(Serialize)]
pub struct SiteContext {
    pub title: String,
    pub tagline: String,
}

#[derive(Serialize, Deserialize)]
pub struct IndexConfig {
    pub title: String,
}

impl BuildContext {
    pub fn output_path(&self, content: impl AsRef<Path>) -> Result<PathBuf> {
        Ok(self
            .output_dir
            .join(content.as_ref().strip_prefix(&self.content_dir)?))
    }

    pub fn page_path(&self, page: impl AsRef<Path>, slug: &str) -> Result<PathBuf> {
        // If this is an index file, if so don't nest in a directory.
        if page.as_ref().file_name().and_then(|f| f.to_str()) == Some("index.md") {
            Ok(self.output_path(page.as_ref())?.with_extension("html"))
        } else {
            Ok(self
                .output_path(page.as_ref())?
                .with_file_name(slug)
                .with_extension("")
                .join("index")
                .with_extension("html"))
        }
    }
}

pub fn build() -> Result<()> {
    let config = std::fs::read_to_string("Config.toml").context("Could not open Config.toml")?;
    let config: SiteConfig = toml::from_str(&config).context("Failed to parse Config.toml")?;

    let output_dir = &config.output_dir;
    // Make sure output directory exists and is empty
    if Path::new(output_dir).exists() {
        std::fs::remove_dir_all(output_dir)?;
    }
    std::fs::create_dir_all(output_dir)?;

    // Copy public stuff from templates
    let mut copy_opts = fs_extra::dir::CopyOptions::new();
    copy_opts.copy_inside = true;
    fs_extra::dir::copy(
        &format!("{}/static", config.template_dir),
        &config.output_dir,
        &copy_opts,
    )?;

    let templates = Tera::new(&format!("{}/**/*.html.tera", config.template_dir))
        .context("Failed to build templates")?;

    let site_context = SiteContext {
        title: config.title,
        tagline: config.tagline,
    };

    let context = BuildContext {
        output_dir: PathBuf::from(&config.output_dir),
        content_dir: PathBuf::from(&config.content_dir),
        templates,
        site_context,
        markdown_renderer: MarkdownRenderer::new(),
    };

    build_directory(&config.content_dir, &context)?;

    println!("Building...");
    Ok(())
}

fn build_directory(dir: impl AsRef<Path>, context: &BuildContext) -> Result<()> {
    println!("Building directory {:?}", dir.as_ref());
    let out_dir = context.output_path(dir.as_ref())?;
    std::fs::create_dir_all(&out_dir)?;

    let mut index = Vec::new();

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            build_directory(entry.path(), context)?;
        } else if metadata.is_file() {
            match entry.path().extension().and_then(|ext| ext.to_str()) {
                Some("md") => index.push(build_page(entry.path(), context)?),
                Some("toml") => {}
                // Copy any other file-types to the output
                // This is useful for things like images
                _ => {
                    let entry_path = entry.path();
                    let out_path = context.output_path(&entry_path)?;
                    println!("Copying {:?} to {:?}", entry_path, out_path);
                    std::fs::copy(&entry_path, &out_path).with_context(|| {
                        format!("Failed to copy file {:?} to {:?}", entry_path, out_path)
                    })?;
                }
            }
        }
    }

    index.sort_by(|a, b| match (a.date, b.date) {
        (Some(a), Some(b)) => b.cmp(&a),
        _ => std::cmp::Ordering::Equal,
    });

    // Build index for pages...
    let index_config = dir.as_ref().join("index.toml");
    if index_config.exists() {
        let index_config = std::fs::read_to_string(index_config)?;
        let index_config: IndexConfig = toml::from_str(&index_config)?;

        let mut template_context = tera::Context::new();
        template_context.insert("pages", &index);
        template_context.insert("index", &index_config);
        template_context.insert("site", &context.site_context);
        let output = context
            .templates
            .render("index.html.tera", &template_context)?;

        std::fs::write(&out_dir.join("index.html"), output)
            .with_context(|| format!("Failed to write page to {:?}", out_dir))?;
    }

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

    let mut template_context = tera::Context::new();
    template_context.insert("content", &page.content);
    template_context.insert("page", &page.metadata);
    template_context.insert("site", &context.site_context);
    let output = context
        .templates
        .render("page.html.tera", &template_context)?;

    std::fs::write(&page.metadata.out_path, output)
        .with_context(|| format!("Failed to write page to {:?}", page.metadata.out_path))?;

    Ok(page.metadata)
}
