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
    fn new() -> &'static mut Self { unsafe {
        SINGLETON = Some(Self {
            appstate: AppState::Sleeping,
            cursor_path: get_cursor_path()
        });

        SINGLETON.as_mut().unwrap()
    } }

    /// Retreives the singleton instance of App. If one
    /// does not already exist, a new one is created and returned. 
    pub fn instance() -> &'static mut Self { unsafe {
        match SINGLETON.as_mut() {
            Some(v) => v,
            None => Self::new()
        }
    } }
    
    // Todo: Should return a Result<(), Error>
    pub fn run(&mut self) {
        println!("Running!");
        Input::poll(Self::on_keystate_changed);
    }

    pub fn activate(&self) {
        // Customize the system cursors to signify that xterminate is active
        let custom_cursor = Cursor::load(self.cursor_path.as_str());
        cursor::set_all(&custom_cursor);
    }

    pub fn deactivate(&self) {
        cursor::reset();
    }

    pub fn xterminate(&self) {
        // Terminate process under the cursor and reset
        // the system cursors back to the default ones.
        cursor::reset();

        let (cursor_x, cursor_y) = cursor::position();
        let target_window = Window::from_point(cursor_x, cursor_y);
        let target_process = target_window.process();

        target_process.terminate();
    }

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
                    self.appstate = AppState::Sleeping;
                    self.xterminate();
                    println!(" Success!");

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

fn get_cursor_path() -> String {
    let rel_cursor_path = std::path::PathBuf::from(CURSOR_FILENAME);
    let mut abs_cursor_path = std::env::current_exe().expect("failed to retrieve path to executable");
    abs_cursor_path.pop(); // Remove executable filename to get the root dir only
    abs_cursor_path.push(rel_cursor_path); // Add the relative cursor path to the root dir

    abs_cursor_path.display().to_string()
}