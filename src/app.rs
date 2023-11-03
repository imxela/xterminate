use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::window::Window;
use crate::input::{Input, KeyCode, KeyStatus, KeyState, Keybind, InputEventHandler};
use crate::tray::{Tray, TrayEvent, TrayEventHandler};
use crate::{cursor, logf};
use crate::cursor::Cursor;
use crate::config::Config;
use crate::registry;

/// The path to the cursor file relative to the executable's working directory
const CURSOR_FILENAME: &str = "cursor.cur";
const ICON_FILENAME: &str = "icon.ico";
const DEFAULT_CONFIG_BYTES: &'static [u8] = include_bytes!("..\\res\\config.toml");
const CONFIG_FILENAME: &str = "config.toml";

#[derive(PartialEq, Eq)]
enum AppState {
    Standby,
    Active,
    Shutdown
}

pub struct App {
    config: Rc<RefCell<Config>>,
    appstate: AppState,
    cursor_path: String,
    keybinds: HashMap<String, Keybind>
}

impl App {
    /// Creates a new singleton instance of `App` and returns it.
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
            cursor_path: get_cursor_path(),
            keybinds
        }))
    }
    
    /// Runs xterminate
    pub fn run(app: Rc<RefCell<Self>>) {
        logf!("Running application");
        
        // Retreive the autostart registry value and if it exists
        // trigger an update in case the exeuctable was moved since
        // last exeuction.
        let autostart_enabled = app.borrow().autostart();
        app.borrow_mut().set_autostart(autostart_enabled);

        logf!("Creating input processor");
        let input = Input::create(app.clone());

        logf!("Creating system tray");
        let tray = Tray::create(&get_icon_path(), app.clone(), app.borrow().keybinds.clone());

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

        keybinds.insert(String::from("terminate_immediate"), Self::keybind_from_config(&config.keybinds.terminate_immediate));
        keybinds.insert(String::from("terminate_click"), Self::keybind_from_config(&config.keybinds.terminate_click));
        keybinds.insert(String::from("terminate_click_confirm"), Self::keybind_from_config(&config.keybinds.terminate_click_confirm));
        keybinds.insert(String::from("terminate_abort"), Self::keybind_from_config(&config.keybinds.terminate_abort));

        keybinds
    }

    /// Creates a [Keybind] from a given keybinding in the [Config].
    fn keybind_from_config(cfg_value: &Vec<String>) -> Keybind {
        let mut keybind = Keybind::empty();

        for key in cfg_value {
            keybind.add(KeyCode::from_string(key.as_str()).expect("config contains an invalid keybind (unrecognized key-code)"));
        }

        keybind
    }

    fn load_config() -> Config {
        let default_config = toml::from_slice::<Config>(DEFAULT_CONFIG_BYTES).unwrap();

        let path = get_resource_path(CONFIG_FILENAME);

        let content = match std::fs::read(&path) {
            Ok(v) => v,
            Err(_e) => {
                logf!("WARNING: No config file found, creating a default one");

                // Create and read the default config
                std::fs::write(&path, DEFAULT_CONFIG_BYTES)
                    .expect("failed to write default config file to drive");

                logf!("Config file created");

                DEFAULT_CONFIG_BYTES.to_vec()
            }
        };
        
        let mut config = toml::from_slice::<Config>(&content)
            .expect("failed to parse config file");

        // Check if the current and new config files are compatible, if not replace the old one.
        if  config.compatibility.version_major < default_config.compatibility.version_major ||
            config.compatibility.version_minor < default_config.compatibility.version_minor ||
            config.compatibility.version_patch < default_config.compatibility.version_patch {
                logf!(
                    "WARNING: Config file compatibility version mismatch, 
                    replacing old config with updated default config 
                    ({}.{}.{}) => {}.{}.{})",
                    config.compatibility.version_major, config.compatibility.version_minor, config.compatibility.version_patch,
                    default_config.compatibility.version_major, config.compatibility.version_minor, config.compatibility.version_patch
                );

                std::fs::write(&path, DEFAULT_CONFIG_BYTES).
                    expect("failed to overwrite old config file");

                config = default_config;

                logf!("Config replaced");
        }

        logf!("Configuration loaded");
        logf!("Config:\n{config:#?}");

        config
    }

    fn save_config(config: &Config) {
        logf!("Writing configuration to disk");

        let path = get_resource_path(CONFIG_FILENAME);

        let content = toml::to_string_pretty::<Config>(config)
            .expect("failed to serialize config");

        std::fs::write(path, content).
            expect("failed to write to config file");

        logf!("Configuration successfully written to disk");
    }

    /// Sets the autostart registry value if `enabled` is true.
    /// If `enabled` is false and an autostart value exists in
    /// the registry, it will be deleted.
    fn set_autostart(&self, enabled: bool) {
        if enabled {
            logf!("Setting registry autostart value");

            registry::set_value(
                registry::HKey::HKeyCurrentUser, 
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", 
                "xterminate", 
                registry::ValueType::Sz, 
                get_executable_path().as_str()
            );

        } else if registry::exists( // Todo: duplicated code, fn autostart() already exists
            registry::HKey::HKeyCurrentUser, 
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", 
            Some("xterminate")
        ) {
            logf!("Deleting registry autostart value");

            registry::delete_value(
                registry::HKey::HKeyCurrentUser, 
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", 
                "xterminate"
            );
        }
    }

    /// Returns true if the autostart registry value exists or false otherwise.
    fn autostart(&self, ) -> bool {
        registry::exists(
            registry::HKey::HKeyCurrentUser,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            Some("xterminate")
        )
    }

    /// Forces the process associated with the specified [Window]
    /// to terminate. If `try_graceful` is true, an attempt will be
    /// made to gracefully exit the window before a termination is made.
    fn terminate(&self, window: &mut Window, try_graceful: bool) {
        let timeout = self.config.borrow().graceful_timeout;

        if try_graceful {
            logf!("Attempting graceful exit, timeout set to {}ms", timeout);

            if window.process().try_exit(timeout) == true {
                logf!("Graceful exit successful");
                return;
            }
        }
        
        logf!("Forcefully terminating");
        window.process().terminate()
    }

    pub fn shutdown(&mut self) {
        logf!("Setting AppState to Shutdown");
        self.appstate = AppState::Shutdown;
    }

    /// Called when going from `AppState::Sleeping` to `ÀppState::Active`.
    /// Sets the system cursors to the xterminate cursor.
    pub fn termination_mode_activate(&mut self) {
        logf!("Termination mode activated by user");
        self.appstate = AppState::Active;

        logf!("Switching to active cursor");
        // Customize the system cursors to signify that xterminate is active
        let custom_cursor = Cursor::load_from_file(self.cursor_path.as_str()).expect("failed to load default cursor from file");
        cursor::set_all(&custom_cursor);
    }

    /// Called when the termination mode is active (AppState = Active) and 
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

    /// Called if the user presses the escape key while in the `AppState::Active` state.
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
        } else {
            logf!("ERROR: failed to terminate foreground window: no valid window is in focus");
            return false;
        }
    }
}

impl InputEventHandler for App {
    fn handle(&mut self, mut state: KeyState, _keycode: KeyCode, _keystatus: KeyStatus) -> bool {
        match self.appstate {
            AppState::Standby => {
                if self.keybinds["terminate_click"].triggered(&mut state) {
                    self.termination_mode_activate();
                    return true;
                } else if self.keybinds["terminate_immediate"].triggered(&mut state) {
                    return self.terminate_active();
                }
            },

            AppState::Active => {
                if self.keybinds["terminate_click_confirm"].triggered(&mut state) {
                    self.termination_mode_confirm();

                    return true;
                } else if self.keybinds["terminate_abort"].triggered(&mut state) {
                    // logf!("Termination aborted by user");
                    // self.appstate = AppState::Standby;
                    // self.deactivate();

                    self.termination_mode_deactivate();

                    return true;
                }
            },

            AppState::Shutdown => {
                // Do nothing
                logf!("Entered shutdown state");
            }
        }

        // No message was processed
        return false;   
    }
}

impl TrayEventHandler for App {
    fn handle(&mut self, event: TrayEvent) {
        match event {
            TrayEvent::OnMenuSelectExit => {
                self.shutdown();
            },

            TrayEvent::OnMenuSelectStartWithWindows => {
                logf!("Setting start with Windows to '{}'", self.autostart());
                self.set_autostart(!self.autostart());
            },

            TrayEvent::OnMenuSelectResetCursor => {
                if self.appstate == AppState::Active {
                    self.appstate = AppState::Standby;
                }
                
                cursor::reset();
            },

            TrayEvent::OnMenuSelectOpenConfig => {
                open_config_file();
            },

            TrayEvent::OnMenuSelectEnterTerminationMode => {
                self.termination_mode_activate();
            }
        }
    }
}

/// Open xterminate's 'config.toml' file for editing in notepad.exe.
pub fn open_config_file() {
    use windows::core::{ PCSTR, PSTR };
    use windows::Win32::Foundation::GetLastError;

    use windows::Win32::System::Threading::{
        CreateProcessA,
        PROCESS_CREATION_FLAGS,
        STARTUPINFOA,
        PROCESS_INFORMATION
    };

    let mut c_filepath = String::from("notepad.exe ");
    c_filepath.push_str(get_resource_path(CONFIG_FILENAME).as_str());
    c_filepath.push('\0');

    let c_notepad_path = "C:\\Windows\\notepad.exe\0";

    let mut si = STARTUPINFOA::default();
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        let result = CreateProcessA(
            PCSTR(c_notepad_path.as_ptr()),
            PSTR(c_filepath.as_mut_ptr()),
            std::ptr::null(),
            std::ptr::null(),
            false,
            PROCESS_CREATION_FLAGS(0),
            std::ptr::null(),
            PCSTR(std::ptr::null()),
            &mut si,
            &mut pi
        ).0 == 1;

        if !result {
            logf!("ERROR: Failed to open config file in notepad.exe (OS Error: {})", GetLastError().0);
        }
    }
}

/// Returns the absolute path of the cursor file.
pub fn get_cursor_path() -> String {
    get_resource_path(CURSOR_FILENAME)
}

pub fn get_icon_path() -> String {
    get_resource_path(ICON_FILENAME)
}

pub fn get_executable_path() -> String {
    std::env::current_exe().expect("failed to get path to executable")
        .display()
        .to_string()
}

/// Returns the absolute path of a file relative to the 'res' folder
/// Equivalent to calling `make_rel_path_abs("res/<filename>")`
pub fn get_resource_path(path: &str) -> String {
    // let relative = std::path::PathBuf::from(format!("res/{}", filename));
    // let mut absolute = std::env::current_exe().expect("failed to get path to executable");
    // absolute.pop(); // Remove the executable filename to get the root directory
    // absolute.push(relative); // Add the relative path to the root directory

    // absolute.display().to_string()

    make_rel_path_abs(format!("res\\{}", path).as_str())
}

/// Returns the absolute path of a file or directory
/// relative to the root application directory.
pub fn make_rel_path_abs(filename: &str) -> String {
    let relative = std::path::PathBuf::from(format!("{}", filename));
    let mut absolute = std::env::current_exe().expect("failed to get path to executable");
    absolute.pop(); // Remove the executable filename to get the root directory
    absolute.push(relative); // Add the relative path to the root directory

    absolute.display().to_string()
}