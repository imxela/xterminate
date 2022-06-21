use std::rc::Rc;
use std::cell::RefCell;

use crate::window::Window;
use crate::input::{Input, KeyCode, KeyStatus, KeyState, InputEventHandler};
use crate::tray::{Tray, TrayEvent, TrayEventHandler};
use crate::cursor;
use crate::cursor::Cursor;

use crate::printfl;

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

    /// Called when the left mouse button is clicked while in the `AppState::Active` state.
    /// Runs the termination procedure and returns true on success or
    /// false if no window is located under the cursor.
    pub fn xterminate(&self) -> bool {
        // Terminate process under the cursor and reset
        // the system cursors back to the default ones.
        cursor::reset();

        let (cursor_x, cursor_y) = cursor::position();
        let target_window = match Window::from_point(cursor_x, cursor_y) {
            Some(v) => v,
            None => return false
        };

        let target_process = target_window.process();

        target_process.terminate();

        true
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
            },

            AppState::Active => {
                if state.pressed(KeyCode::LeftMouseButton) {
                    println!(" Triggered!");
                    printfl!("Terminating...");
                    if !self.xterminate() {
                        println!(" Failed (no window at mouse position, trying again)");
                        return false;
                    }

                    println!(" Success!");
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

        return false;   
    }
}

impl TrayEventHandler for App {
    fn handle(&mut self, event: TrayEvent) {
        match event {
            TrayEvent::OnMenuSelectExit => {
                self.shutdown();
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

/// Returns the absolute path of a file relative to the 'res' folder
fn get_resource_path(filename: &str) -> String {
    let relative = std::path::PathBuf::from(format!("res/{}", filename));
    let mut absolute = std::env::current_exe().expect("failed to get path to executable");
    absolute.pop(); // Remove the executable filename to get the root directory
    absolute.push(relative); // Add the relative path to the root directory

    absolute.display().to_string()
}