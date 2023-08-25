#![windows_subsystem = "windows"]

pub mod app;
pub mod error;
pub mod input;
pub mod process;
pub mod window;
pub mod cursor;
pub mod tray;
pub mod config;
pub mod registry;
pub mod logger;

use app::App;

fn main() {
    logf!(
        "Environment information:\nOS: {}\nApplication: {}", 
        os_info::get(), 
        env!("CARGO_PKG_VERSION")
    );

    error::set_panic_hook();
    
    App::run(App::new());
}