pub use std::error::Error;

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

use crate::input;

pub fn display_error_dialog(human_message: &str) {
    unsafe {
        MessageBoxA(HWND(0), human_message, "xterminate.exe", MB_OK | MB_ICONERROR);
    }
}

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(on_panic));
}

fn on_panic(info: &std::panic::PanicInfo) {
    // Ensures that the MessageBox events are not captured
    // by the application since it's in the middle of a panic
    // and would not be able to process them
    input::unregister();

    let message: String;

    match info.payload().downcast_ref::<String>() {
        Some(panic_string) => {
            message = format!("Unexpected error: {}.\n\nPanic Information:\n\n{:#?}\n\nSorry. :(", panic_string, info);
        },

        None => {
            message = format!("An unexpected error ocurred but no additional error information was supplied.\n\nPanic Information:\n\n{:#?}\n\nSorry. :(", info);
        }
    }

    eprintln!("{}", message);

    unsafe {
        MessageBoxA(HWND(0), message, "Panic! in xterminate", MB_OK | MB_ICONERROR);
    }
}