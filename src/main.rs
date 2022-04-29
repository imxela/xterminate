pub mod app;
pub mod input;
pub mod process;
pub mod window;
pub mod cursor;

macro_rules! printfl {
    ($($arg:tt)*) => {
        use std::io::Write;

        print!("{}", format_args!($($arg)*));
        std::io::stdout().flush().unwrap();
    };
}

pub(crate) use printfl;

use app::App;

fn main() {
    let app = App::instance();
    app.run();
}