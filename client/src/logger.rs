//! A logger that prints all messages in browser's console.

use log;
use wasm_bindgen::JsValue;
use web_sys::console;

use log::{Level, Log, Metadata, Record, SetLoggerError};

pub struct Config {
    pub level: Level,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            level: Level::Trace,
        }
    }
}

static LOGGER: WebLogger = WebLogger;

struct WebLogger;

impl Log for WebLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO Check the args of a location
        true
    }

    fn log(&self, record: &Record) {
        let metadata = record.metadata();
        if self.enabled(metadata) {
            let msg = format!(
                "{}:{} -- {}",
                record.level(),
                record.target(),
                record.args()
            );
            let msgs = js_sys::Array::new();
            msgs.push(&JsValue::from_str(&msg));
            match metadata.level() {
                Level::Trace => console::trace(&msgs),
                Level::Debug => console::debug(&msgs),
                Level::Info => console::info(&msgs),
                Level::Warn => console::warn(&msgs),
                Level::Error => console::error(&msgs),
            }
        }
    }

    fn flush(&self) {}
}

pub fn try_init(config: Config) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    let level = config.level;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

pub fn init() {
    try_init(Config::default())
        .expect("web_logger::init should not be called after logger initialized");
}

pub fn custom_init(config: Config) {
    try_init(config)
        .expect("web_logger::custom_init should not be called after logger initialized");
}
