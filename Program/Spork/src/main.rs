mod api;

mod terminal;
mod gui;

use iced::Application;

use std::env;

static mut IP: String = String::new();

fn main() -> iced::Result {
    let mut arg = "".to_string();
    if let Some(arg1) = env::args().nth(1) {
        arg = arg1;
    }

    if &arg == "-c" {
        terminal::terminal::start_console();
        std::process::exit(0);
    }

    gui::start::Spork::run(iced::Settings::default())
}
