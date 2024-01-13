#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::module_name_repetitions)]
#![windows_subsystem = "windows"]

pub mod app;
pub mod config;
pub mod cursor;
pub mod error;
pub mod input;
pub mod logger;
pub mod process;
pub mod registry;
pub mod tray;
pub mod window;

use app::App;

fn main() {
    logf!(
        "Environment information:\nOS: {}\nApplication: {}",
        os_info::get(),
        env!("CARGO_PKG_VERSION")
    );

    error::set_panic_hook();

    App::run(&App::new());
}
