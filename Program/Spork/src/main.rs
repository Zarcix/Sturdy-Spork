use std::io::Read;

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
    

    let client = reqwest::blocking::Client::new();

    window.printw("Enter TV IP: ");
    window.keypad(true);

    unsafe {
        IP = user_input(&window);
    }
    
    let result = launch_wvc(unsafe {&IP}, &client);

    if result.is_err() {
        error_reset(&window,  result.unwrap_err())
    }

    main_menu_print(&window, &client);
}

fn user_input(window: &pancurses::Window) -> String {
    let mut user_input = String::new();
    while let Some(input) = window.getch() {
        match input {
            pancurses::Input::KeyBackspace => {
                if user_input.is_empty() {
                    window.addch(' ');
                } else {
                    window.delch();
                }

                window.refresh();

                user_input.pop();
            }
            pancurses::Input::Character(input) => {
                if input == '\n' {
                    break;
                }
                user_input.push(input);
            }
            _ => {}
        }
    }

    user_input
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

fn launch_wvc(url: &str, client: &reqwest::blocking::Client) -> Result<(), String> {
    log::debug!("Pinging Device...");
    let response = client.get(format!("http://{}:8060/query/apps", &url)).send();
    match response {
        Err(a) => {
            return Result::Err(a.to_string());
        }

        _ => {}
    }
    log::debug!("Found Roku Device! Attempting to launch Web Video Caster...");
    let response = response.unwrap().text().unwrap();

    let doc = roxmltree::Document::parse(&response).unwrap();
    let elem = doc.descendants().find(|n| n.text() == Some("Web Video Caster - Receiver"));
    match elem {
        None => {
            return Result::Err("Web Video Caster not installed".to_string());
        }

        _ => {}
    }
    let elem = elem.unwrap().attribute("id").unwrap();

    log::debug!("Found Web Video Caster on the Roku Device with ID: {}. Attempting to launch...", elem);

    client.post(format!("http://{}:8060/launch/{}", &url, &elem)).send().unwrap();

    log::debug!("Launched Web Video Caster on the Roku Device!");
    
    return Result::Ok(());

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
        main_menu_parse(window, client);
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

fn main_menu_parse(window: &pancurses::Window, client: &reqwest::blocking::Client) {
    while let Some(input) = window.getch() {
        match input {
            // Media Controls
            pancurses::Input::KeyLeft => {
                let ip = unsafe{&IP};
                client.post(format!("http://{ip}:8060/keypress/Left")).send().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(500));
                client.post(format!("http://{ip}:8060/keypress/Play")).send().unwrap();
            }
            pancurses::Input::KeyRight => {
                let ip = unsafe{&IP};
                client.post(format!("http://{ip}:8060/keypress/Right")).send().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(500));
                client.post(format!("http://{ip}:8060/keypress/Play")).send().unwrap();
            }
            pancurses::Input::KeySRight => {
                let ip = unsafe{&IP};
                client.post(format!("http://{ip}:8060/keypress/Fwd")).send().unwrap();
            }
            pancurses::Input::KeySLeft => {
                let ip = unsafe{&IP};
                client.post(format!("http://{ip}:8060/keypress/Rev")).send().unwrap();
            }
            pancurses::Input::Character(' ') => {
                pause(client);
                return
            }

            // Application Controls
            pancurses::Input::Character('\u{1b}') => { // Esc Control
                // Relocate user back to home
                let ip = unsafe {&IP};
                client.post(format!("http://{ip}:8060/keypress/Home")).send().unwrap();

                pancurses::endwin();
                std::process::exit(0);
            }

            // Queueing Videos
            pancurses::Input::Character('q') => {
                window.clear();
                play_video(window, client);
                return
            }

            // Refresh on any other key
            _ => {
                return
            }
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

    let playvid_status = start_video(&vid_url, &extension, client);

    if playvid_status.is_err() {
        let err = playvid_status.unwrap_err();
        window.clear();
        window.printw(err);
        play_video(window, client);
    }

}

fn pause(client: &reqwest::blocking::Client) {
    let ip = unsafe {&IP};
    client.post(format!("http://{ip}:8060/keypress/Play")).send().unwrap();
}
fn start_video(input_url: &String, extension: &String, client: &reqwest::blocking::Client) -> Result<(), String> {
    if !input_url.contains("http") || !input_url.contains("://"){
        return Err("Not a valid URL".to_string());
    }
    let extension = FILETYPES.get(&extension);

    if extension.is_none() {
        return Err("Not a valid extension choice".to_string());
    }

    let extension = extension.unwrap().to_owned();

    let encoded_url = urlencoding::encode(&input_url);

    let ip = unsafe {&IP};

    client.post(format!("http://{ip}:8060/input?cmd=play&url={encoded_url}&pos=0&tit=Video&sub=false&media={}&fmt={}", extension.0, extension.1)).send().unwrap();
    
    return Ok(())
}