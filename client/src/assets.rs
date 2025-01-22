use std::path::Path;

pub struct Asset(&'static str);

macro_rules! asset {
    ($file:expr $(,)?) => {{
        let _ = include_bytes!($file);
        Asset($file)
    }};
}

impl Asset {
    pub fn path(&self) -> String {
        Path::new("/src/")
            .join(self.0)
            .to_string_lossy()
            .split("/static/")
            .nth(1)
            .map(|path| format!("/{}", path))
            .unwrap()
    }
}

pub const GITHUB_LOGO: Asset = asset!("../static/assets/img/github.svg");
