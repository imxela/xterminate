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

/// Flushed `print!()` macro
macro_rules! printfl {
    ($($arg:tt)*) => {
        use std::io::Write;

        print!("{}", format_args!($($arg)*));
        std::io::stdout().flush().unwrap();
    };
}

/// Flushed `eprint!()` macro
macro_rules! eprintfl {
    ($($arg:tt)*) => {
        use std::io::Write;

        eprint!("{}", format_args!($($arg)*));
        std::io::stderr().flush().unwrap();
    };
}

pub(crate) use printfl;
pub(crate) use eprintfl;

use app::App;

fn main() {
    error::set_panic_hook();
    
    App::run(App::new());
}