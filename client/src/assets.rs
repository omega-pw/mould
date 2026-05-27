use crate::SharedString;
use leptos::prelude::*;
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
    pub fn image_view(&self, style: Option<SharedString>) -> impl IntoView {
        let path = self.path();
        view! {
            <img src={path} style={style}/>
        }
    }
}

pub const GITHUB_LOGO: Asset = asset!("../static/assets/img/github.svg");
pub const LOGIN_BG: Asset = asset!("../static/assets/img/login-bg.svg");
