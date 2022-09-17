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

fn main() {
    env_logger::init();

    let client = reqwest::blocking::Client::new();
    let mut URL = String::new();

    
    println!("Enter the IP of the Roku Device: ");
    std::io::stdin().read_line(&mut URL).expect("Bad Input.");
    URL.pop();

    launch_wvc(&URL, &client);

    start_video(&URL, &client);

}

fn launch_wvc(url: &str, client: &reqwest::blocking::Client) {
    log::debug!("Pinging Device...");
    let response = client.get(format!("http://{}:8060/query/apps", &url)).send();
    match response {
        Err(_) => {
            println!("IP is not a Roku Device\n");
            main();
            return;
        }

        _ => {}
    }
    log::debug!("Found Roku Device! Attempting to launch Web Video Caster...");
    let response = response.unwrap().text().unwrap();

    let doc = roxmltree::Document::parse(&response).unwrap();
    let elem = doc.descendants().find(|n| n.text() == Some("Web Video Caster - Receiver"));
    match elem {
        None => {
            println!("Device does not have Web Video Caster installed. Closing");
            main();
            return;
        }

        _ => {}
    }
    let elem = elem.unwrap().attribute("id").unwrap();

    log::debug!("Found Web Video Caster on the Roku Device with ID: {}. Attempting to launch...", elem);

    client.post(format!("http://{}:8060/launch/{}", &url, &elem)).send().unwrap();

    log::debug!("Launched Web Video Caster on the Roku Device!");

}

fn start_video(url: &str, client: &reqwest::blocking::Client) {
    let mut input_url = String::new();
    println!("Enter a URL for the video or image you are about to play:");

    std::io::stdin().read_line(&mut input_url).expect("Bad Input");
    input_url.pop();
    if !input_url.contains("http") || !input_url.contains("://"){
        println!("Not a valid HTTP request.");
        start_video(url, client);
    }

    println!("Enter the file extension for the previous input. Supported extensions are:\n");
    for (key, value) in &FILETYPES {
        println!("{}: {}", value.0, key);
    }
    let mut extension = String::new();
    std::io::stdin().read_line(&mut extension).expect("Bad Input");
    extension.pop();

    let extension = *FILETYPES.get(&extension).expect("Extension Not Found");

    let encoded_url = urlencoding::encode(&input_url);
    client.post(format!("http://{url}:8060/input?cmd=play&url={encoded_url}&pos=0&tit=Video&sub=false&media={}&fmt={}", extension.0, extension.1)).send().unwrap();
}