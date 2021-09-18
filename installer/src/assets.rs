use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "build-assets"]
pub struct BuildAssets;
