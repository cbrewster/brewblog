use serde_derive::Deserialize;

/// Global site configuration
#[derive(Deserialize, Debug)]
pub struct SiteConfig {
    /// Title of the site
    pub title: String,

    /// Site's tagline
    pub tagline: String,

    /// Site's domain
    pub domain: String,

    /// Directory to output generated site
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Directory where pages are stored
    #[serde(default = "default_content_dir")]
    pub content_dir: String,
}

fn default_output_dir() -> String {
    "public".into()
}

fn default_content_dir() -> String {
    "content".into()
}
