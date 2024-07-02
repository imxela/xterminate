#![allow(clippy::multiple_crate_versions)]
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
pub mod ui;
pub mod updater;
pub mod window;

use app::App;

fn main() {
    logf!(
        "Environment information:\nOS: {}\nApplication: {}",
        os_info::get(),
        env!("CARGO_PKG_VERSION")
    );

    error::set_panic_hook();

    if instance_count() > 1 {
        // An instance is already running
        ui::taskdialog::TaskDialog::new()
            .set_title("Already running")
            .set_heading("Already running")
            .set_content("An instance of xterminate is already running. Please exit the running instance before starting a new one.")
            .set_icon(ui::taskdialog::TaskDialogIcon::InformationIcon)
            .display()
            .result();

        return;
    }

    App::run(&App::new());
}

/// Returns the amount of processes with the same file name as the current one.
fn instance_count() -> u32 {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };

    let mut count = 0;

    unsafe {
        let mut process_entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>().try_into().unwrap(),
            ..Default::default()
        };

        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();

        let mut result = Process32First(snapshot, &mut process_entry);

        loop {
            if result.is_ok() {
                if process_entry.szExeFile.starts_with(
                    std::env::current_exe()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .as_encoded_bytes(),
                )
                // The rest should be nuls
                {
                    count += 1;
                }
            } else {
                break;
            }

            result = Process32Next(snapshot, &mut process_entry);
        }
    }

    count
}
