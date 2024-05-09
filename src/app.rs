use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::UI::Shell::{FOLDERID_ProgramData, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

use crate::config::Config;
use crate::cursor::Cursor;
use crate::input::{Input, KeyCode, KeyState, KeyStatus, Keybind};
use crate::process::ExitMethod;
use crate::registry;
use crate::tray::{Tray, TrayEvent};
use crate::ui::taskdialog::TaskDialog;
use crate::window::Window;
use crate::{cursor, logf};

/// The path to the cursor file relative to the executable's working directory
const CURSOR_FILENAME: &str = "cursor.cur";
const ICON_FILENAME: &str = "icon.ico";
const DEFAULT_CONFIG_BYTES: &[u8] = include_bytes!("..\\res\\config.toml");
const CONFIG_FILENAME: &str = "config.toml";
const LOGFILES_PATH: &str = "logs\\";

#[derive(PartialEq, Eq)]
enum AppState {
    Standby,
    Active,
    Shutdown,
}

pub struct App {
    config: Rc<RefCell<Config>>,
    appstate: AppState,
    cursor_path: String,
    keybinds: HashMap<String, Keybind>,
}

impl App {
    /// Creates a new singleton instance of [`App`] and returns it.
    #[must_use]
    pub fn new() -> Rc<RefCell<Self>> {
        crate::logger::initialize();

        logf!("Creating application instance");

        logf!("Loading application configuration");
        let config = Rc::new(RefCell::new(Self::load_config()));

        logf!(
            "Application configuration version: {}.{}.{}",
            config.borrow().compatibility.version_major,
            config.borrow().compatibility.version_major,
            config.borrow().compatibility.version_major
        );

        logf!("Setting up keybinds");
        let keybinds = Self::setup_keybinds(&mut config.borrow_mut());

        logf!("Application instance created successfully");

        Rc::new(RefCell::new(Self {
            config,
            appstate: AppState::Standby,
            cursor_path: cursor_path(),
            keybinds,
        }))
    }

    #[allow(clippy::never_loop)]
    /// Runs xterminate
    pub fn run(app: &Rc<RefCell<Self>>) {
        logf!("Running application");

        // Retrieve the autostart registry value and if it exists
        // trigger an update in case the exeuctable was moved since
        // last exeuction.
        let autostart_enabled = Self::autostart();
        Self::set_autostart(autostart_enabled);

        logf!("Creating input processor");
        let input = Input::create(app.clone());

        logf!("Creating system tray");
        let tray = Tray::create(&icon_path(), app.clone(), app.borrow().keybinds.clone());

        logf!("Starting event loop");
        while app.borrow().appstate != AppState::Shutdown {
            // The message loops for input and tray both run
            // on the same thread so we can use WaitMessage()
            // to block the thread until a message is receieved
            // instead of wasting CPU time on polling constantly.
            use windows::Win32::UI::WindowsAndMessaging::WaitMessage;
            unsafe { WaitMessage() };

            input.borrow().poll();
            tray.borrow().poll();
        }

        logf!("Exited event loop, saving config and freeing resources");
        Self::save_config(&app.borrow_mut().config.borrow_mut());
        input.borrow().unregister();
        tray.borrow().delete();

        logf!("Goodbye");
    }

    fn setup_keybinds(config: &mut Config) -> HashMap<String, Keybind> {
        let mut keybinds: HashMap<String, Keybind> = HashMap::new();

        keybinds.insert(
            String::from("terminate_immediate"),
            Self::keybind_from_config(&config.keybinds.terminate_immediate),
        );
        keybinds.insert(
            String::from("terminate_click"),
            Self::keybind_from_config(&config.keybinds.terminate_click),
        );
        keybinds.insert(
            String::from("terminate_click_confirm"),
            Self::keybind_from_config(&config.keybinds.terminate_click_confirm),
        );
        keybinds.insert(
            String::from("terminate_abort"),
            Self::keybind_from_config(&config.keybinds.terminate_abort),
        );

        keybinds
    }

    /// Creates a [`Keybind`] from a given keybinding in the [`Config`].
    fn keybind_from_config(cfg_value: &Vec<String>) -> Keybind {
        let mut keybind = Keybind::empty();

        for key in cfg_value {
            keybind.add(
                KeyCode::from_string(key.as_str())
                    .expect("config contains an invalid keybind (unrecognized key-code)"),
            );
        }

        keybind
    }

    fn load_config() -> Config {
        let default_config = toml::from_slice::<Config>(DEFAULT_CONFIG_BYTES).unwrap();

        let path = config_path();

        let content = match std::fs::read(&path) {
            Ok(v) => v,
            Err(_e) => {
                logf!("WARNING: No config file found, creating a default one");

                if !appdata_path().exists() {
                    std::fs::create_dir_all(appdata_path())
                        .expect("failed to create xterminate program data directory");
                }

                // Create and read the default config
                std::fs::write(&path, DEFAULT_CONFIG_BYTES)
                    .expect("failed to write default config file to drive");

                logf!("Config file created");

                DEFAULT_CONFIG_BYTES.to_vec()
            }
        };

        let mut config = toml::from_slice::<Config>(&content).expect("failed to parse config file");

        // Check if the current and new config files are compatible, if not replace the old one.
        if config.compatibility.version_major < default_config.compatibility.version_major
            || config.compatibility.version_minor < default_config.compatibility.version_minor
            || config.compatibility.version_patch < default_config.compatibility.version_patch
        {
            logf!(
                "WARNING: Config file compatibility version mismatch, 
                    replacing old config with updated default config 
                    ({}.{}.{}) => {}.{}.{})",
                config.compatibility.version_major,
                config.compatibility.version_minor,
                config.compatibility.version_patch,
                default_config.compatibility.version_major,
                config.compatibility.version_minor,
                config.compatibility.version_patch
            );

            std::fs::write(&path, DEFAULT_CONFIG_BYTES)
                .expect("failed to overwrite old config file");

            config = default_config;

            logf!("Config replaced");
        }

        logf!("Configuration loaded");
        logf!("Config:\n{config:#?}");

        config
    }

    fn save_config(config: &Config) {
        logf!("Writing configuration to disk");

        let path = config_path();

        let content = toml::to_string_pretty::<Config>(config).expect("failed to serialize config");

        std::fs::write(path, content).expect("failed to write to config file");

        logf!("Configuration successfully written to disk");
    }

    /// Sets the autostart registry value if `enabled` is true.
    /// If `enabled` is false and an autostart value exists in
    /// the registry, it will be deleted.
    fn set_autostart(enabled: bool) {
        if enabled {
            logf!("Setting registry autostart value");

            registry::set_value(
                registry::HKey::HKeyCurrentUser,
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "xterminate",
                registry::ValueType::Sz,
                executable_path().as_str(),
            );
        } else if registry::exists(
            // Todo: duplicated code, fn autostart() already exists
            registry::HKey::HKeyCurrentUser,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            Some("xterminate"),
        ) {
            logf!("Deleting registry autostart value");

            registry::delete_value(
                registry::HKey::HKeyCurrentUser,
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                "xterminate",
            );
        }
    }

    /// Returns true if the autostart registry value exists or false otherwise.
    fn autostart() -> bool {
        registry::exists(
            registry::HKey::HKeyCurrentUser,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            Some("xterminate"),
        )
    }

    /// Forces the process associated with the specified [Window]
    /// to terminate. If `try_graceful` is true, an attempt will be
    /// made to gracefully exit the window before a termination is made.
    fn terminate(&self, window: &mut Window, try_graceful: bool) {
        // Divide by 3 since we try 3 different exit methods and
        // we want the timeout to be the total waiting time.
        let timeout = self.config.borrow().graceful_timeout / 3u32;

        let target_process = &mut window.process();

        logf!("Will close process {}", target_process);

        if try_graceful {
            logf!(
                "Attempting graceful exit methods with timeout set to {}ms",
                timeout
            );

            if !(target_process.try_exit(&ExitMethod::Close, timeout)
                || target_process.try_exit(&ExitMethod::Destroy, timeout)
                || target_process.try_exit(&ExitMethod::Quit, timeout))
            {
                logf!("Graceful exit failed");
            }
        } else {
            logf!("Graceful exit disabled ");
        }

        logf!("Terminating forcefully");
        target_process.terminate();
    }

    pub fn shutdown(&mut self) {
        logf!("Setting AppState to Shutdown");
        self.appstate = AppState::Shutdown;
    }

    /// Called when going from [`AppState::Standby`] to [`AppState::Active`].
    /// Sets the system cursors to the xterminate cursor.
    ///
    /// # Panics
    /// Will panic if loading the system cursor fails.
    /// See [`windows::Windows::Win32::UI::WindowsAndMessaging::LoadImageA`].
    pub fn termination_mode_activate(&mut self) {
        logf!("Termination mode activated by user");
        self.appstate = AppState::Active;

        logf!("Switching to active cursor");
        // Customize the system cursors to signify that xterminate is active
        let custom_cursor = Cursor::load_from_file(self.cursor_path.as_str())
            .expect("failed to load default cursor from file");
        cursor::set_all(&custom_cursor);
    }

    /// Called when the termination mode is active ([`Self::appstate`] == [`AppState::Active`]) and
    /// the confirmation keybind is pressed by the user. This will trigger
    /// termination of the window the mouse cursor is currently hovering over.
    pub fn termination_mode_confirm(&mut self) {
        logf!("Termination confirmed by user");

        // Terminate process under the cursor and reset
        // the system cursors back to the default ones.
        cursor::reset();

        let (cursor_x, cursor_y) = cursor::position();
        if let Some(window) = &mut Window::from_point(cursor_x, cursor_y) {
            self.terminate(window, self.config.borrow().attempt_graceful);
            logf!("Terminated successfully");
        } else {
            logf!("ERROR: Failed to terminate: no window under mouse pointer");
        }

        self.appstate = AppState::Standby;
    }

    /// Called if the user presses the escape key while in the [`AppState::Active`] state.
    /// Resets the cursor back to system defaults.
    pub fn termination_mode_deactivate(&mut self) {
        logf!("Termination aborted by user");
        self.appstate = AppState::Standby;

        logf!("Switching to normal cursor");
        cursor::reset();
    }

    /// Called when the user presses the immediate/active termination keybind
    /// or triggers it manually from the tray menu. Responsible for terminating
    /// the currently focused window.
    pub fn terminate_active(&mut self) -> bool {
        logf!("Immediate termination triggered by user");

        if let Some(window) = &mut Window::from_foreground() {
            self.terminate(window, self.config.borrow().attempt_graceful);
            logf!("Terminated successfully");
            return true;
        }

        logf!("ERROR: failed to terminate foreground window: no valid window is in focus");
        false
    }
}

impl crate::input::EventHandler for App {
    fn handle(&mut self, mut state: KeyState, _keycode: KeyCode, _keystatus: KeyStatus) -> bool {
        match self.appstate {
            AppState::Standby => {
                if self.keybinds["terminate_click"].triggered(&mut state) {
                    self.termination_mode_activate();
                    return true;
                } else if self.keybinds["terminate_immediate"].triggered(&mut state) {
                    return self.terminate_active();
                }
            }

            AppState::Active => {
                if self.keybinds["terminate_click_confirm"].triggered(&mut state) {
                    self.termination_mode_confirm();

                    return true;
                } else if self.keybinds["terminate_abort"].triggered(&mut state) {
                    self.termination_mode_deactivate();

                    return true;
                }
            }

            AppState::Shutdown => {
                // Do nothing
                logf!("Entered shutdown state");
            }
        }

        // No message was processed
        false
    }
}

impl crate::tray::TrayEventHandler for App {
    fn handle(&mut self, event: TrayEvent) {
        match event {
            TrayEvent::OnMenuSelectExit => {
                self.shutdown();
            }

            TrayEvent::OnMenuSelectStartWithWindows => {
                logf!("Setting start with Windows to '{}'", Self::autostart());
                Self::set_autostart(!Self::autostart());
            }

            TrayEvent::OnMenuSelectResetCursor => {
                if self.appstate == AppState::Active {
                    self.appstate = AppState::Standby;
                }

                cursor::reset();
            }

            TrayEvent::OnMenuSelectOpenConfig => {
                open_config_file();
            }

            TrayEvent::OnMenuSelectEnterTerminationMode => {
                self.termination_mode_activate();
            }

            TrayEvent::OnMenuSelectAbout => {
                TaskDialog::new()
                    .set_title("About")
                    .set_heading(format!("xterminate v{}", env!("CARGO_PKG_VERSION")))
                    .set_content(
                        "Easily terminate any windowed process by the press of a button.\
                        \n\nThis software was created by <a href=\"https://github.com/imxela\">@imxela</a> and is licensed under the open-source \
                        <a href=\"https://github.com/imxela/xterminate/blob/main/LICENSE\">MIT license</a>. \
                        The source code is publicly available in xterminate's <a href=\"https://github.com/imxela/xterminate\">GitHub repository</a>.\n\n\
                        Contact information can be found on my <a href=\"https://xela.me\">website</a>.\
                        \n\nThank you for using my software! <3"
                    )
                    .set_hyperlinks_enabled(true)
                    .display_blocking();
            }
        }
    }
}

/// Open xterminate's 'config.toml' file for editing in notepad.exe.
pub fn open_config_file() {
    use windows::core::{PCSTR, PSTR};
    use windows::Win32::Foundation::GetLastError;

    use windows::Win32::System::Threading::{
        CreateProcessA, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOA,
    };

    let mut c_filepath = String::from("notepad.exe ");
    c_filepath.push_str(&config_path());
    c_filepath.push('\0');

    let c_notepad_path = "C:\\Windows\\notepad.exe\0";

    let si = STARTUPINFOA::default();
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        let result = CreateProcessA(
            PCSTR(c_notepad_path.as_ptr()),
            PSTR(c_filepath.as_mut_ptr()),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            PCSTR(std::ptr::null()),
            &si,
            &mut pi,
        )
        .0 == 1;

        if !result {
            logf!(
                "ERROR: Failed to open config file in notepad.exe (OS Error: {})",
                GetLastError().0
            );
        }
    }
}

/// Returns the absolute path of the cursor file.
#[must_use]
pub fn cursor_path() -> String {
    resource_path(CURSOR_FILENAME)
}

#[must_use]
pub fn icon_path() -> String {
    resource_path(ICON_FILENAME)
}

#[must_use]
pub fn config_path() -> String {
    make_rel_appdata_path_abs(CONFIG_FILENAME)
        .display()
        .to_string()
}

#[must_use]
pub fn logfiles_path() -> String {
    make_rel_appdata_path_abs(LOGFILES_PATH)
        .display()
        .to_string()
}

#[must_use]
/// # Panics
///
/// Panics if the underlying call to [`std::env::current_exe()`] does.
pub fn executable_path() -> String {
    std::env::current_exe()
        .expect("failed to get path to executable")
        .display()
        .to_string()
}

/// Returns xterminate's data directory (%APPDATA%/xterminate)
///
/// # Panics
///
/// Panics if the underlying call to [`SHGetKnownFolderPath()`] fails or if
/// the [`PWSTR`] returned by said function cannot be turned into a [`String`].
#[must_use]
pub fn appdata_path() -> std::path::PathBuf {
    unsafe {
        let appdata_path =
            SHGetKnownFolderPath(&FOLDERID_ProgramData, KF_FLAG_DEFAULT, HANDLE::default())
                .expect("failed to get application data path");

        let mut appdata_path = std::path::PathBuf::from(appdata_path.to_string().unwrap());
        appdata_path.push("xterminate");

        appdata_path
    }
}

/// Returns the absolute path of a file or directory relative
/// to xterminate's application data directory.
#[must_use]
pub fn make_rel_appdata_path_abs(filename: &str) -> std::path::PathBuf {
    debug_assert!(
        !filename.starts_with('/') && !filename.starts_with('\\'),
        "argument `filename` is relative and cannot start with a '/' or '\\' character"
    );

    let mut appdata_path = appdata_path();
    appdata_path.push(filename);

    appdata_path
}

/// Returns the absolute path of a file relative to the 'res' folder
/// Equivalent to calling [`make_rel_path_abs("res/<filename>")`]
#[must_use]
pub fn resource_path(path: &str) -> String {
    make_rel_path_abs(format!("res\\{path}").as_str())
}

/// Returns the absolute path of a file or directory
/// relative to the root application directory.
///
/// # Panics
///
/// Panics if the underlying call to [`std::env::current_exe()`] fails.
#[must_use]
pub fn make_rel_path_abs(filename: &str) -> String {
    debug_assert!(
        !filename.starts_with('/') && !filename.starts_with('\\'),
        "argument `filename` is relative and cannot start with a '/' or '\\' character"
    );

    let relative = std::path::PathBuf::from(filename);
    let mut absolute = std::env::current_exe().unwrap();
    absolute.pop(); // Remove the executable filename to get the root directory
    absolute.push(relative); // Add the relative path to the root directory

    absolute.display().to_string()
}
