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
        window.printw("Welcome to the Main Menu
        Avaiable Options:
        (Q)ueue a new video
        (Space) to toggle pause
        (Esc) to quit
        ");

        main_menu_parse(window, client);
    }
}

fn main_menu_parse(window: &pancurses::Window, client: &reqwest::blocking::Client) {
    while let Some(input) = window.getch() {
        match input {
            pancurses::Input::Character('q') => {
                window.clear();
                play_video(window, client);
                return
            }

            pancurses::Input::Character(' ') => {
                pause(client);
                return
            }

            pancurses::Input::Character('\u{1b}') => {
                pancurses::endwin();
                std::process::exit(0);
            }
            _ => {}
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