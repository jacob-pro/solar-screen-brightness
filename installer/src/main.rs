use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "build-assets"]
struct BuildAssets;

fn main() {
    for file in BuildAssets::iter() {
        println!("{}", file.as_ref());
    }
}
