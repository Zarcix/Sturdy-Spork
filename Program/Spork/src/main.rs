use std::io::Read;
mod api;
use phf::phf_map;

static FILETYPES: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    "mp4" => ("video", "mp4"),
    "mkv" => ("video", "x-matroska"),
    "m3u8" =>("video", "hls"),
    "mov" => ("video", "quicktime"),

    "png" => ("image", "png"),
    "jpg" => ("image", "jpg"),
};

static mut IP: String = String::new();

fn main() {
    let result = env_logger::try_init();
    let window = pancurses::initscr();

    window.nodelay(true);
    
    ctrlc::set_handler(move || {
        pancurses::endwin();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let client = reqwest::blocking::Client::new(); // Need to make this an ArcMutex

    window.printw("Enter TV IP: ");
    window.keypad(true);

    unsafe {
        IP = user_input(&window);
    }
    
    let result = {
        let client = client.clone();
        std::thread::spawn(|| {
            api::tv_calls::WVCLaunch(client)
        }).join().unwrap()
    };

    if result.is_err() {
        error_reset(&window,  result.unwrap_err())
    }

    main_menu_print(&window, &client);
}

fn user_input(window: &pancurses::Window) -> String {
    window.nodelay(false);
    let mut user_input = String::new();
    loop {
        match window.getch() {
            Some(pancurses::Input::KeyBackspace) => {
                if user_input.is_empty() {
                    window.addch(' ');
                } else {
                    window.delch();
                }

                window.refresh();

                user_input.pop();
            }
            Some(pancurses::Input::Character(input)) => {
                if input == '\n' {

                    window.nodelay(true);
                    return user_input
                }
                user_input.push(input);
            }
            Some(_) => (),
            None => { 
                window.nodelay(true);
                return user_input 
            }
        }
    }
}

fn error_reset(window: &pancurses::Window, err: String) { // Generates main menu 
    window.printw(format!("Error: {}. Press 'Enter' to continue.", err));

        while let Some(input) = window.getch() {
            match input {
                pancurses::Input::Character(input) => {
                    if input == '\n' {
                        window.clear();
                        pancurses::endwin();
                        main();
                        return;
                    }
                }
                _ => {}
            }
        }
}

fn main_menu_print(window: &pancurses::Window, client: &reqwest::blocking::Client) {
    loop {
        window.clear();
        window.printw("Esc to quit");
        let (max_y,max_x) = window.get_max_yx();

        let control_win = window.subwin(8, max_x, max_y - 8, 0).unwrap();
        control_win.draw_box(0, 0);


        control_win_print(&control_win, client);


        window.refresh();

        let getch = window.getch();

        main_menu_parse(window, client, getch);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn control_win_print(window: &pancurses::Window, client: &reqwest::blocking::Client) {
    //TODO need to update this with time

    // Reverse
    window.mvprintw(4, 1, "<--");
    window.mvprintw(5, 1, "<<-");
    window.mvprintw(6, 1, "<<<");

    // Space
    window.mvprintw(4, window.get_max_x() / 2 - 7, "Pause");
    window.mvprintw(5, window.get_max_x() / 2 - 7, "Space");
    window.mvprintw(6, window.get_max_x() / 2 - 7, "Pause");

    // Fast-Forward
    window.mvprintw(4, window.get_max_x() - 4, "-->");
    window.mvprintw(5, window.get_max_x() - 4, "->>");
    window.mvprintw(6, window.get_max_x() - 4, ">>>");

    let stats = vid_status(client);
    window.printw(format!("{:?}", stats));
}

fn vid_status(client: &reqwest::blocking::Client) -> ((i32, i32), i32) {

    let ip = unsafe {&IP};
    let status = client.get(format!("http://{ip}:8060/query/media-player")).send().unwrap().text().unwrap();
    let mut times = (-1, -1);
    let is_playing = is_playing(&status);

    if is_playing != 0 {
        times = time_status(&status);
    }

    return (times, is_playing)

}

fn is_playing(req: &String) -> i32 {
    let split: Vec<&str> = req.split("state=\"").collect();
    let string = split[1].to_string();
    let split: Vec<&str> = string.split("\">").collect();
    let string = split[0];

    if string == "play" { return 1 }
    if string == "pause" { return 2 }

    return 0;
}

fn time_status(req: &String) -> (i32, i32) {
    let split: Vec<&str> = req.split("<position>").collect();
    let string = split[1].to_string();
    let split: Vec<&str> = string.split(" ms</position>").collect();
    let mut currentTime = split[0].parse::<i32>().unwrap();
    currentTime = currentTime / 1000;


    let split: Vec<&str> = req.split("<duration>").collect();
    let string = split[1].to_string();
    let split: Vec<&str> = string.split(" ms</duration>").collect();
    let mut maxTime = split[0].parse::<i32>().unwrap();
    maxTime = maxTime / 1000;

    return (currentTime, maxTime)
}

fn main_menu_parse(window: &pancurses::Window, client: &reqwest::blocking::Client, getch: Option<pancurses::Input>) {
    if getch.is_none() {
        return
    }

    let getch = getch.unwrap();
    let client = client.clone();
    match getch {
        // Media Controls
        pancurses::Input::KeyLeft => {
            { // Skip 5 backwards
                std::thread::spawn(move || {
                    api::tv_calls::TVLeft(client)
                });
            }
        }
        pancurses::Input::KeyRight => {
            { // Skip 5 foward
                std::thread::spawn(move || {
                    api::tv_calls::TVRight(client)
                });
            }
        }
        pancurses::Input::KeySRight => {
            { // Fast Foward
                std::thread::spawn(move || {
                    api::tv_calls::TVFwd(client);
                });
            }
        }
        pancurses::Input::KeySLeft => {
            { // Reverse
                std::thread::spawn(move || {
                    api::tv_calls::TVRev(client);
                });
            }
        }
        pancurses::Input::Character(' ') => {
            { // TV Play/Pause
                std::thread::spawn(move || {
                    api::tv_calls::TVTogglePause(client)
                });
            }
            return
        }

        // Application Controls
        pancurses::Input::Character('\u{1b}') => { // Esc Control
            // Relocate user back to home
            { // TV Home
                std::thread::spawn(move || {
                    api::tv_calls::TVHome(client);
                });
            }

            pancurses::endwin();
            std::process::exit(0);
        }

        // Queueing Videos
        pancurses::Input::Character('q') => {
            window.clear();
            play_video(window, &client);
            return
        }

        // Refresh on any other key
        _ => {
            return
        }
    }
}

fn play_video(window: &pancurses::Window, client: &reqwest::blocking::Client) {
    window.printw("Enter the URL that you want to play:\n");
    window.refresh();

    let vid_url = user_input(window);


    window.clear();
    window.printw("Please enter a file extension. Supposed extensions are:");
    for (key, value) in &FILETYPES {
        window.printw(format!("{}: {}\n", value.0, key));
    }
    window.printw("Input: ");
    let extension = user_input(window);

    {
        let client = client.clone();
        std::thread::spawn(move || {
            api::tv_calls::WVCPlay(client, &vid_url.to_owned(), &extension.to_owned())
        });
    }

}
