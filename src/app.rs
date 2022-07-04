use std::rc::Rc;
use std::cell::RefCell;

use crate::window::Window;
use crate::input::{Input, KeyCode, KeyStatus, KeyState, InputEventHandler};
use crate::tray::{Tray, TrayEvent, TrayEventHandler};
use crate::cursor;
use crate::cursor::Cursor;
use crate::registry;

use crate::printfl;
use crate::eprintfl;

/// The path to the cursor file relative to the executable's working directory
const CURSOR_FILENAME: &str = "cursor.cur";
const ICON_FILENAME: &str = "icon.ico";

#[derive(PartialEq, Eq)]
enum AppState {
    Standby,
    Active,
    Shutdown
}

pub struct App {
    appstate: AppState,
    cursor_path: String
}

impl App {
    /// Creates a new singleton instance of `App` and returns it.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { 
            appstate: AppState::Standby, 
            cursor_path: get_cursor_path()
        }))
    }
    
    /// Runs xterminate
    pub fn run(app: Rc<RefCell<Self>>) {
        println!("Running!");
        
        // Retreive the autostart registry value and if it exists
        // trigger an update in case the exeuctable was moved since
        // last exeuction.
        let autostart_enabled = app.borrow().autostart();
        app.borrow_mut().set_autostart(autostart_enabled);

        let input = Input::create(app.clone());
        let tray = Tray::create(&get_icon_path(), app.clone());

        while app.borrow().appstate != AppState::Shutdown {
            // The message loops for input and tray both run
            // on the same thread so we can use WaitMessage()
            // to block the thread until a message is receieved
            // instead of wasting CPU time on polling constantly.
            use windows::Win32::UI::WindowsAndMessaging::WaitMessage;
            unsafe { WaitMessage() };

            input.poll();
            tray.poll();
        }

        printfl!("Exiting...");
        input.unregister();
        tray.delete();
        println!(" Done!");
    }


    /// Sets the autostart registry value if `enabled` is true.
    /// If `enabled` is false and an autostart value exists in
    /// the registry, it will be deleted.
    fn set_autostart(&self, enabled: bool) {
        if enabled {
            registry::set_value(
                registry::HKey::HKeyCurrentUser, 
                "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", 
                "xterminate", 
                registry::ValueType::Sz, 
                get_executable_path().as_str()
            );
        } else if registry::exists(
            registry::HKey::HKeyCurrentUser, 
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", 
            Some("xterminate")
        ) {
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
        // Todo: Get timout from config
        let timeout = 3500;
        
        if try_graceful {
            println!("Attempting graceful exit, timeout set to {}ms", timeout);

            if window.process().try_exit(timeout) == true {
                println!("Graceful exit sucessful!");
                return;
            }
        }
        
        println!("Graceful exit failed, terminating.");
        window.process().terminate()
    }

    pub fn shutdown(&mut self) {
        self.appstate = AppState::Shutdown;
    }

    /// Called when going from `AppState::Sleeping` to `ÀppState::Active`.
    /// Sets the system cursors to the xterminate cursor.
    pub fn activate(&self) {
        // Customize the system cursors to signify that xterminate is active
        let custom_cursor = Cursor::load_from_file(self.cursor_path.as_str()).expect("failed to load default cursor from file");
        cursor::set_all(&custom_cursor);
    }

    /// Called if the user presses the escape key while in the `AppState::Active` state.
    /// Resets the cursor back to system defaults.
    pub fn deactivate(&self) {
        cursor::reset();
    }
}

impl InputEventHandler for App {
    fn handle(&mut self, mut state: KeyState, _keycode: KeyCode, _keystatus: KeyStatus) -> bool {
        match self.appstate {
            AppState::Standby => { 
                if state.pressed(KeyCode::LeftControl) &&
                   state.pressed(KeyCode::LeftAlt) &&
                   state.pressed(KeyCode::End) {
                        println!("Activated!");
                        printfl!("Waiting for trigger...");

                        self.appstate = AppState::Active;
                        self.activate();

                        return true;
                } 
                else if state.pressed(KeyCode::LeftControl) &&
                        state.pressed(KeyCode::LeftAlt) &&
                        state.pressed(KeyCode::F4) {
                            if let Some(window) = &mut Window::from_foreground() {
                                self.terminate(window, true);
                                return true;
                            } else {
                                eprintln!("failed to terminate foreground window: no valid window is in focus");
                                return false;
                            }
                }
            },

            AppState::Active => {
                if state.pressed(KeyCode::LeftMouseButton) {
                    println!(" Triggered!");
                    printfl!("Terminating...");

                    // Terminate process under the cursor and reset
                    // the system cursors back to the default ones.
                    cursor::reset();

                    let (cursor_x, cursor_y) = cursor::position();
                    if let Some(window) = &mut Window::from_point(cursor_x, cursor_y) {
                        self.terminate(window, true);
                        printfl!(" Success!");
                    } else {
                        eprintfl!(" Failed to terminate window: no window under mouse pointer");
                    }

                    self.appstate = AppState::Standby;
                    return true;
                } else if state.pressed(KeyCode::Escape) {
                    printfl!("Aborted.");
                    self.appstate = AppState::Standby;
                    self.deactivate();

                    return true;
                }
            },

            AppState::Shutdown => {
                // Do nothing
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
                self.set_autostart(!self.autostart());
                println!("Start with Windows set to '{}'", self.autostart());
            },

            TrayEvent::OnMenuSelectResetCursor => {
                if self.appstate == AppState::Active {
                    self.appstate = AppState::Standby;
                }
                
                cursor::reset();
            }
        }
    }
}

/// Returns the absolute path of the cursor file.
fn get_cursor_path() -> String {
    get_resource_path(CURSOR_FILENAME)
}

fn get_icon_path() -> String {
    get_resource_path(ICON_FILENAME)
}

fn get_executable_path() -> String {
    std::env::current_exe().expect("failed to get path to executable")
        .display()
        .to_string()
}

/// Returns the absolute path of a file relative to the 'res' folder
fn get_resource_path(filename: &str) -> String {
    let relative = std::path::PathBuf::from(format!("res/{}", filename));
    let mut absolute = std::env::current_exe().expect("failed to get path to executable");
    absolute.pop(); // Remove the executable filename to get the root directory
    absolute.push(relative); // Add the relative path to the root directory

    absolute.display().to_string()
}