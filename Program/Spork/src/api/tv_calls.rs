use phf::phf_map;
use reqwest::blocking::Client;

static FILETYPES: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    "mp4" => ("video", "mp4"),
    "mkv" => ("video", "x-matroska"),
    "m3u8" =>("video", "hls"),
    "mov" => ("video", "quicktime"),

    "png" => ("image", "png"),
    "jpg" => ("image", "jpg"),
};

/// TV Controls

pub fn TVLeft(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/Left")).send().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(400));
    client.post(format!("http://{ip}:8060/keypress/Play")).send().unwrap();
}

pub fn TVRight(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/Right")).send().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(400));
    client.post(format!("http://{ip}:8060/keypress/Play")).send().unwrap();
}

pub fn TVFwd(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/Fwd")).send().unwrap();
}

pub fn TVRev(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/Rev")).send().unwrap();
}

pub fn TVVolUp(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/VolumeUp")).send().unwrap();
}

pub fn TVVolDown(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/VolumeDown")).send().unwrap();
}

pub fn TVVolMute(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/VolumeMute")).send().unwrap();
}

pub fn TVTogglePause(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/Play")).send().unwrap();
}

pub fn TVHome(client: Client) {
    let ip = unsafe { crate::IP.clone() };
    client.post(format!("http://{ip}:8060/keypress/Home")).send().unwrap();
}

pub fn TVMedia(client: Client) -> ((i32, i32), i32) {
    let ip = unsafe { crate::IP.clone() };

    
    let status = client.get(format!("http://{ip}:8060/query/media-player")).send().unwrap().text().unwrap();
    let mut times = (-1, -1);
    let mut isPlaying = 0;

    // Get Play Status
    let split: Vec<&str> = status.split("state=\"").collect();
    let string = split[1].to_string();
    let split: Vec<&str> = string.split("\">").collect();
    let string = split[0];

    if string == "play" {isPlaying = 1}
    else if string == "pause" {isPlaying = 2}
    
    // Get Timing Status
    if isPlaying != 0 {
        let split: Vec<&str> = status.split("<position>").collect();
        let string = split[1].to_string();
        let split: Vec<&str> = string.split(" ms</position>").collect();
        let currentTime = split[0].parse::<i32>().unwrap() / 1000;

        let split: Vec<&str> = status.split("<duration>").collect();
        let string = split[1].to_string();
        let split: Vec<&str> = string.split(" ms</duration>").collect();
        let maxTime = split[0].parse::<i32>().unwrap() / 1000;
        times = (currentTime, maxTime);
    }


    return (times, isPlaying)
}

pub fn WVCPlay(client: Client, url: &String, media_type: &String) {
    let ip = unsafe { crate::IP.clone() };


    if !url.contains("http") || !url.contains("://") { // Check for https:// or http://
        return;
    }

    let extension = FILETYPES.get(&media_type);

    if extension.is_none() {
        return;
    }

    let extension = extension.unwrap();

    let encoded_url = urlencoding::encode(&url);

    client.post(format!("http://{ip}:8060/input?cmd=play&url={encoded_url}&pos=0&tit=Video&sub=false&media={}&fmt={}", extension.0, extension.1)).send().unwrap();

}

// Launch WVC on TV
pub fn WVCLaunch(client: Client) -> Result<(), String> {
    let ip = unsafe { crate::IP.clone() };
    
    let response = client.get(format!("http://{}:8060/query/apps", &ip)).send();
    match response {
        Err(_a) => {
            return Err("Cannot contact provided IP".to_string());
        }

        _ => {}
    }
    let response = response.unwrap().text().unwrap();

    let doc = roxmltree::Document::parse(&response).unwrap();
    let elem = doc.descendants().find(|n| n.text() == Some("Web Video Caster - Receiver"));
    match elem {
        None => {
            return Err("WVC Not Found".to_string());
        }

        _ => {}
    }
    let elem = elem.unwrap().attribute("id").unwrap();

    client.post(format!("http://{}:8060/launch/{}", &ip, &elem)).send().unwrap();

    return Ok(())
}
