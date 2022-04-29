pub mod app;
pub mod input;
pub mod process;
pub mod window;
pub mod cursor;

#[macro_use]
extern crate lazy_static;

use app::App;

fn main() {
    let app = App::instance();
    app.run();
}