use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "build-assets"]
pub struct BuildAssets;

#[derive(RustEmbed)]
#[folder = "../assets"]
pub struct AppAssets;
