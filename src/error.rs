pub use std::error::Error;

use crate::ui::taskdialog::{self, TaskDialog};
use crate::{app, logf};

/// Represents a human-readable error including an optional error code (e.g. system error code)
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct AppError {
    message: String,
    code: Option<usize>,
    _inner: Option<Box<dyn Error>>,
}

impl AppError {
    #[must_use]
    pub fn new(message: &str, code: Option<usize>, inner: Option<Box<dyn Error>>) -> Self {
        Self {
            message: message.to_owned(),
            code,
            _inner: inner,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.code {
            Some(v) => write!(f, "{} (error code {:#06x}).", self.message, v),
            None => write!(f, "{}", self.message),
        }
    }
}

impl Error for AppError {}

// Shorthand
pub type AppResult<T> = Result<T, AppError>;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR, MB_OK};

use chrono::Local;
use windows::core::PCSTR;

/// # Panics
///
/// Panics if `human_message` can not be turned into a valid [`CString`].
pub fn display_error_dialog(human_message: &str) {
    unsafe {
        MessageBoxA(
            HWND(0),
            PCSTR(
                std::ffi::CString::new(human_message)
                    .unwrap()
                    .as_bytes()
                    .as_ptr(),
            ),
            PCSTR("xterminate.exe\0".as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(on_panic));
}

fn on_panic(info: &std::panic::PanicInfo) {
    let message: String;

    let date = Local::now().format("%Y-%m-%d");
    let timestamp = Local::now().format("%H:%M:%S.%f");

    let backtrace_directory = crate::app::make_rel_path_abs("logs");
    let backtrace_filepath =
        app::make_rel_path_abs(format!("logs/{date}-{timestamp}.backtrace").as_str());

    std::fs::create_dir_all(backtrace_directory).unwrap();

    match info.payload().downcast_ref::<String>() {
        Some(panic_string) => {
            message = format!("Unexpected error: {panic_string}.\n\nPanic Information:\n\n{info:#?}\n\nBacktrace saved to '{backtrace_filepath}'.\n\nSorry. :(");
        }

        None => match info.payload().downcast_ref::<&str>() {
            Some(panic_str) => {
                message = format!("Unexpected error: {panic_str}.\n\nPanic Information:\n\n{info:#?}\n\nBacktrace saved to '{backtrace_filepath}'.\n\nSorry. :(");
            }

            None => {
                message = format!("An unexpected error ocurred but no additional error information was supplied.\n\nPanic Information:\n\n{info:#?}\n\nBacktrace saved to '{backtrace_filepath}'.\n\nSorry. :(");
            }
        },
    }

    logf!("PANIC: {}", message);

    TaskDialog::new()
        .set_title("Panic! in xterminate")
        .set_heading("An error occured in xterminate")
        .set_content(message)
        .set_icon(taskdialog::TaskDialogIcon::ErrorIcon)
        .display_blocking();
}
