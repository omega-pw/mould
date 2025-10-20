use std::env;
use tihu::SharedString;

pub struct Arguments {
    pub config_path: String,
}

impl Arguments {
    pub fn try_from_args() -> Result<Arguments, SharedString> {
        let mut args = Vec::new();
        for v in env::args() {
            args.push(v);
        }
        let config_path = args
            .get(1)
            .ok_or_else(|| SharedString::from_static("app require 1 parameter!"))?;
        return Ok(Arguments {
            config_path: config_path.clone(),
        });
    }
}
