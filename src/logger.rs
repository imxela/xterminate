extern crate chrono;

use chrono::Local;

use std::io::Write;

/// Formatted logging macro
#[macro_export]
macro_rules! logf {
    ($($arg:tt)*) => {
        crate::logger::log(
            format!("{}", format_args!($($arg)*)),
            file!(),
            line!()
        );
    };
}

pub fn initialize() {
    // Delete logfiles dated over 7 days in the past

    // 1. Open log folder
    // 2. Enumerate log files
    // 3. For each file, parse date
    // 4. If date exceeds 7 days ago, delete the file
    // 5. Once all files have been iterated, init is complete

    let logfile_directory = format!("{}", crate::app::make_rel_path_abs("logs"));
    std::fs::create_dir_all(logfile_directory)
        .unwrap();

    let entries = std::fs::read_dir(crate::app::make_rel_path_abs("logs"))
        .unwrap();

    for entry in entries {
        let entry_path = entry.unwrap().path();
        let entry_filename = entry_path.file_name().unwrap();
        let log_date = std::path::Path::new(entry_filename).file_stem().unwrap();

        let date = chrono::NaiveDate::parse_from_str(log_date.to_str().unwrap(), "%Y-%m-%d").unwrap();
        if date.checked_add_days(chrono::Days::new(7)).unwrap() < Local::now().naive_local().date() {
            // Log if older than 7 days, delete
            std::fs::remove_file(entry_path)
                .unwrap();
        }
    }
}

pub fn log(message: String, file: &'static str, line: u32) {
    let date = Local::now().format("%Y-%m-%d");
    let timestamp = Local::now().format("%H:%M:%S.%f");

    let formatted_message = format!(
        "[{0}] [{1}:{2}] {3}",
        timestamp,
        file,
        line,
        message
    );

    let mut space_count = 0;

    for c in formatted_message.chars() {
        if c == '\n' {
            space_count += 1;
        }
    }

    let mut spaces = String::new();

    for _ in 0..space_count {
        spaces += " ";
    }

    let formatted_message = formatted_message.replace('\n', "\n\t");

    // Todo: Check severity and use stderr when appropriate
    println!("{formatted_message}");

    let logfile_directory = format!("{}", crate::app::make_rel_path_abs("logs"));
    let log_filepath = format!("{}{}", logfile_directory, format!("\\{}.log", date));

    std::fs::create_dir_all(logfile_directory)
        .unwrap();

    let mut log_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(log_filepath)
        .unwrap();

    writeln!(log_file, "{}", formatted_message)
        .expect("failed to write to log file");
}