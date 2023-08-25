pub use std::error::Error;

use crate::{logf, app};

/// Represents a human-readable error including an optional error code (e.g. system error code)
#[derive(Debug)]
pub struct AppError {
    message: String, 
    code: Option<usize>,
    _inner: Option<Box<dyn Error>>
}

impl AppError {
    pub fn new(message: &str, code: Option<usize>, inner: Option<Box<dyn Error>>) -> Self {
        Self {
            message: message.to_owned(),
            code,
            _inner: inner
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.code {
            Some(v) => write!(f, "{} (error code {:#06x}).", self.message, v),
            None => write!(f, "{}", self.message)
        }
    }
}

impl Error for AppError {}

// Shorthand
pub type AppResult<T> = Result<T, AppError>;

use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_OK, MB_ICONERROR};
use windows::Win32::Foundation::HWND;

use chrono::Local;

pub fn display_error_dialog(human_message: &str) {
    unsafe {
        MessageBoxA(HWND(0), human_message, "xterminate.exe", MB_OK | MB_ICONERROR);
    }
}

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(on_panic));
}

fn on_panic(info: &std::panic::PanicInfo) {
    let message: String;

    let date = Local::now().format("%Y-%m-%d");
    let timestamp = Local::now().format("%H:%M:%S.%f");
    
    let backtrace_directory = format!("{}", crate::app::make_rel_path_abs("logs"));
    let backtrace_filepath = app::make_rel_path_abs(format!("logs/{}-{}.backtrace", date, timestamp).as_str());

    std::fs::create_dir_all(backtrace_directory)
        .unwrap();

    match info.payload().downcast_ref::<String>() {
        Some(panic_string) => {
            message = format!("Unexpected error: {}.\n\nPanic Information:\n\n{:#?}\n\nBacktrace saved to '{}'.\n\nSorry. :(", panic_string, info, backtrace_filepath);
        },

        None => {
            match info.payload().downcast_ref::<&str>() {
                Some(panic_str) => {
                    message = format!("Unexpected error: {}.\n\nPanic Information:\n\n{:#?}\n\nBacktrace saved to '{}'.\n\nSorry. :(", panic_str, info, backtrace_filepath);
                },
        
                None => {
                    message = format!("An unexpected error ocurred but no additional error information was supplied.\n\nPanic Information:\n\n{:#?}\n\nBacktrace saved to '{}'.\n\nSorry. :(", info, backtrace_filepath);
                }
            }
        }
    }

    logf!("PANIC: {}", message);

    unsafe {
        MessageBoxA(HWND(0), message, "Panic! in xterminate", MB_OK | MB_ICONERROR);
    }
}