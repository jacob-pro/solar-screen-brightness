fn main() {
    if cfg!(windows) {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon-256.ico");
        res.compile().unwrap();
    }
}
