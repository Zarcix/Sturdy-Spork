mod api;

mod terminal;

use iced::Application;

use std::env;

static mut IP: String = String::new();

fn main() {
    let mut arg = "".to_string();
    if let Some(arg1) = env::args().nth(1) {
        arg = arg1;
    }

    terminal::terminal::start_console();
}
