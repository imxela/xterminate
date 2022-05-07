use crate::input::{Input, KeyCode, KeyStatus, KeyState};
use crate::cursor;
use crate::cursor::Cursor;
use crate::window::Window;

use crate::printfl;

static mut SINGLETON: Option<App> = None;

/// The path to the cursor file relative to the executable's working directory
const CURSOR_FILENAME: &str = "data/cursor.cur";

enum AppState {
    Sleeping,
    Active
}

pub struct App {
    appstate: AppState,
    cursor_path: String
}

impl App {
    /// Creates a new singleton instance of `App` and returns it.
    fn new() -> &'static mut Self { unsafe {
        SINGLETON = Some(Self {
            appstate: AppState::Sleeping,
            cursor_path: get_cursor_path()
        });

        SINGLETON.as_mut().unwrap()
    } }

    /// Retrieves the singleton instance of App. If one
    /// does not already exist, a new one is created and returned. 
    pub fn instance() -> &'static mut Self { unsafe {
        match SINGLETON.as_mut() {
            Some(v) => v,
            None => Self::new()
        }
    } }
    
    /// Runs xterminate
    pub fn run(&mut self) {
        println!("Running!");
        Input::poll(Self::on_keystate_changed)
    }

    /// Called when going from `AppState::Sleeping` to `ÀppState::Active`.
    /// Sets the system cursors to the xterminate cursor.
    pub fn activate(&self) {
        // Customize the system cursors to signify that xterminate is active
        let custom_cursor = Cursor::load_from_file(get_cursor_path().as_str()).expect("failed to load default cursor from file");
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

    /// Callback invoked by the `Input` struct when the keystate changes.
    /// Responsible for transitioning between different states.
    fn on_keystate_changed(&mut self, state: KeyState, _keycode: KeyCode, _keystatus: KeyStatus) -> bool {
        match self.appstate {
            AppState::Sleeping => { 
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
                    self.appstate = AppState::Sleeping;
                    return true;
                } else if state.pressed(KeyCode::Escape) {
                    printfl!("Aborted.");
                    self.appstate = AppState::Sleeping;
                    self.deactivate();

                    return true;
                }
            }
        }

        return false;
    }
}

/// Returns the absolute path of the cursor file.
fn get_cursor_path() -> String {
    let rel_cursor_path = std::path::PathBuf::from(CURSOR_FILENAME);
    let mut abs_cursor_path = std::env::current_exe().expect("failed to retrieve path to executable");
    abs_cursor_path.pop(); // Remove executable filename to get the root dir only
    abs_cursor_path.push(rel_cursor_path); // Add the relative cursor path to the root dir

    abs_cursor_path.display().to_string()
}