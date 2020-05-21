//! Reads in a page and outputs markdown and metadata

use crate::build::BuildContext;
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use serde_derive::Deserialize;
use std::path::{Path, PathBuf};

const META_TAG: &'static str = "@Meta";

const CONTENT_TAG: &'static str = "@Content";

#[derive(Debug)]
pub struct Page {
    pub metadata: PageMetadata,
    pub content: String,
}

/// Metadata about this page
#[derive(Debug)]
pub struct PageMetadata {
    /// Title of the page
    pub title: String,

    /// Slug of the page, filename by default.
    pub slug: String,

    /// Date the page was posted
    pub date: Option<NaiveDate>,

    /// Link to the page
    pub link: String,

    /// Output file path
    pub out_path: PathBuf,
}

// The metadata to parse from the file.
// This will be merged into the `PageMetadata` with extra info
#[derive(Deserialize, Debug)]
struct Metadata {
    title: String,
    slug: Option<String>,
    date: Option<NaiveDate>,
}

impl Page {
    pub fn parse(path: impl AsRef<Path>, context: &BuildContext) -> Result<Page> {
        let path = path.as_ref();

        let file_contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to open page: {:?}", path))?;

        let meta_line = match file_contents.find(META_TAG) {
            Some(index) => index,
            None => return Err(anyhow!("Could not find @Meta section in page: {:?}", path)),
        };

        let content_line = match file_contents.find(CONTENT_TAG) {
            Some(index) => index,
            None => {
                return Err(anyhow!(
                    "Could not find @Content section in page: {:?}",
                    path
                ))
            }
        };

        let file_meta = &file_contents[(meta_line + META_TAG.len())..content_line];
        let file_meta: Metadata = toml::from_str(file_meta)
            .with_context(|| format!("Failed to parse metadata on page: {:?}", path))?;

        let file_name = match path.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => return Err(anyhow!("Failed to get file name for path: {:?}", path)),
        };

        let metadata = PageMetadata {
            title: file_meta.title,
            date: file_meta.date,
            slug: file_meta.slug.unwrap_or(file_name),
            link: "todo".into(),
            out_path: context.page_path(path)?,
        };

        let content = file_contents[(content_line + CONTENT_TAG.len())..].into();
        dbg!(&content);

        Ok(Page { metadata, content })
    }
}
