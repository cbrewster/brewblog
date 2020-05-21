use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SiteConfig {
    title: String,
    tagline: String,
    domain: String,
}
