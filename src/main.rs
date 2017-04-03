#![recursion_limit = "1024"]

extern crate reqwest;
extern crate bmp;
extern crate rand;
extern crate toml;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

use std::io::prelude::*;
// use std::result::Result;
// use std::error::Error;
use std::path::Path;

#[derive(Deserialize, Debug)]
struct ConfigData {
    image: ConfigImage,
}

#[derive(Deserialize, Debug, Clone)]
struct ConfigUserToml {
    users: Vec<ConfigUser>,
}

#[derive(Deserialize, Debug, Clone)]
struct ConfigUser {
    username: String,
    password: String,
}
#[derive(Deserialize, Debug, Clone)]
struct ConfigImage {
    path: String,
    offset: ConfigOffset,
}
#[derive(Deserialize, Debug, Clone, Copy)]
struct ConfigOffset {
    x: u32,
    y: u32,
}

#[derive(Deserialize, Debug)]
struct RedditLogin {
    json: RedditLoginJson,
}
#[derive(Deserialize, Debug)]
struct RedditLoginJson {
    data: RedditLoginData,
}
#[derive(Deserialize, Debug)]
struct RedditLoginData {
    modhash: String,
}
#[derive(Deserialize, Debug)]
struct RedditDraw {
    wait_seconds: i32,
}
#[derive(Deserialize, Debug)]
struct RedditPixel {
    x: u32,
    y: u32,
    timestamp: f64,
    user_name: String,
    color: u32,
}


mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! { 
        // Automatic conversions between this error chain and other
        // error chains. In this case, it will e.g. generate an
        // `ErrorKind` variant called `Another` which in turn contains
        // the `other_error::ErrorKind`, with conversions from
        // `other_error::Error`.
        //
        // Optionally, some attributes can be added to a variant.
        //
        // This section can be empty.
        links {
            // Another(other_error::Error, other_error::ErrorKind) #[cfg(unix)];
        }

        // Automatic conversions between this error chain and other
        // error types not defined by the `error_chain!`. These will be
        // wrapped in a new error with, in the first case, the
        // `ErrorKind::Fmt` variant. The description and cause will
        // forward to the description and cause of the original error.
        //
        // Optionally, some attributes can be added to a variant.
        //
        // This section can be empty.
        foreign_links {
            Config(::toml::de::Error);
            Request(::reqwest::Error);
            Fmt(::std::fmt::Error);
            Io(::std::io::Error);
        }

        // Define additional `ErrorKind` variants. The syntax here is
        // the same as `quick_error!`, but the `from()` and `cause()`
        // syntax is not supported.
        errors {
            // LoginError(t: String) {
            //     description("cannot login to reddit API")
            //     display("cannot login to reddit API: '{}'", t)
            // }
        }        
    }
}

use errors::*;


struct UserToken {
    cookies: Vec<String>,
    modhash: String,
    username: String,
}

/// Login to Reddit.
/// return a UserToken object which can be passed to function draw
fn login<'a>(username: &'a str, password: &'a str) -> Result<UserToken> {
    println!("[login] username: {}", username);

    let client = reqwest::Client::new()?;
    let url = format!("https://www.reddit.com/api/login/{}", username);
    let params =
        [("op", "login"), ("user", &username), ("passwd", &password), ("api_type", "json")];
    let result = client.post(&url)
        .form(&params)
        .send();

    let mut response = result?;

    // get cookie & modhash
    let body: RedditLogin = response.json()?;
    let modhash = body.json.data.modhash;
    let cookies = match response.headers().get::<reqwest::header::SetCookie>(){
        Some(cookies) => cookies,
        None => bail!("cookie seem missing"),
    };

    // println!("modhash: {:?}", modhash);
    // println!("cookie: {:?}", cookies);

    Ok(UserToken {
        cookies: cookies.0.clone(),
        modhash: modhash,
        username: username.to_string(),
    })
}

fn draw(user_token: &UserToken, x: u32, y: u32, color: u32) {
    println!("[paint] user: {}, coordinate: ({}, {}), color: {}",
             user_token.username,
             x,
             y,
             color);

    let client = reqwest::Client::new().unwrap();
    let url = "https://www.reddit.com/api/place/draw.json";
    let params = [("x", x), ("y", y), ("color", color)];
    let mut headers = reqwest::header::Headers::new();
    headers.set(reqwest::header::Cookie(user_token.cookies.clone()));
    headers.append_raw("x-modhash", user_token.modhash.as_bytes().to_vec());

    let result = client.post(url)
        .form(&params)
        .headers(headers)
        .send();

    match result {
        Ok(mut response) => {
            let status_code = response.status().clone();
            match status_code {
                reqwest::StatusCode::Ok => {
                    match response.json::<RedditDraw>() {
                        Ok(body) => {
                            // println!("  body: {:?}", body);
                            let wait_seconds = body.wait_seconds;
                            println!("  wait_seconds: {}", wait_seconds);
                        }
                        Err(why) => {
                            println!("  json result is error: {}", why);
                        }
                    }
                }
                _ => {
                    println!("  status code is error: {}", response.status());
                }
            }
        }
        Err(why) => println!("  error when sending request: {}", why),
    }
}

// fn download_bitmap(){
//     println!("[bitmap] check bitmap in /r/place");

//     let client = reqwest::Client::new().unwrap();
//     let url = "https://www.reddit.com/api/place/board-bitmap";
//     let result = client.get(url).send();
//     let mut response = result.unwrap();

//     let mut buffer: Vec<u8> = Vec::with_capacity(1000 * 1000);
//     std::io::copy(&mut response, &mut buffer).expect("Failed to read response");
//     println!("buffer: {:?}", &buffer[0 .. 20]);
// }


/// return true when same, false othewise
fn check_pixel(x: u32, y: u32, color: u32) -> bool {
    println!("[pixel check] is coordinate ({},{}) == {} ?", x, y, color);

    let client = reqwest::Client::new().unwrap();
    let url = format!("https://www.reddit.com/api/place/pixel.json?x={}&y={}",
                      x,
                      y);
    let result = client.get(&url).send();
    let mut response = result.unwrap();

    // use std::io;
    // use std::io::prelude::*;
    // let mut buffer = String::new();
    // response.read_to_string(&mut buffer);
    // println!("buffer: {:?}", buffer);

    let body: RedditPixel = response.json().unwrap();

    if body.color != color {
        println!("  NO (current color: {} by {}). need redraw...",
                 body.color,
                 body.user_name);
        return false;
    } else {
        println!("  YES");
        return true;
    }
}

fn load_image(path: &str) -> (u32, u32, Vec<u32>) {
    let palletes: [(u8, u8, u8); 16] = [(255, 255, 255),
                                        (228, 228, 228),
                                        (136, 136, 136),
                                        (34, 34, 34),
                                        (255, 167, 209),
                                        (229, 0, 0),
                                        (229, 149, 0),
                                        (160, 106, 66),
                                        (229, 217, 0),
                                        (148, 224, 68),
                                        (2, 190, 1),
                                        (0, 211, 221),
                                        (0, 131, 199),
                                        (0, 0, 234),
                                        (207, 110, 228),
                                        (130, 0, 128)];

    // <div style="background-color: rgb(255, 255, 255);" class="place-swatch"></div>
    // <div style="background-color: rgb(228, 228, 228);" class="place-swatch"></div>
    // <div style="background-color: rgb(136, 136, 136);" class="place-swatch"></div>
    // <div style="background-color: rgb(34, 34, 34);" class="place-swatch"></div>
    // <div style="background-color: rgb(255, 167, 209);" class="place-swatch"></div>
    // <div style="background-color: rgb(229, 0, 0);" class="place-swatch"></div>
    // <div style="background-color: rgb(229, 149, 0);" class="place-swatch"></div>
    // <div style="background-color: rgb(160, 106, 66);" class="place-swatch"></div>
    // <div style="background-color: rgb(229, 217, 0);" class="place-swatch"></div>
    // <div style="background-color: rgb(148, 224, 68);" class="place-swatch"></div>
    // <div style="background-color: rgb(2, 190, 1);" class="place-swatch"></div>
    // <div style="background-color: rgb(0, 211, 221);" class="place-swatch"></div>
    // <div style="background-color: rgb(0, 131, 199);" class="place-swatch"></div>
    // <div style="background-color: rgb(0, 0, 234);" class="place-swatch"></div>
    // <div style="background-color: rgb(207, 110, 228);" class="place-swatch"></div>
    // <div style="background-color: rgb(130, 0, 128);" class="place-swatch"></div></div>

    println!("[load reference bitmap] {}", path);

    let img = bmp::open(path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    let width = img.get_width();
    let height = img.get_height();

    let mut content: Vec<u32> = Vec::new();

    for (x, y) in img.coordinates() {
        let pixel = img.get_pixel(x, y);
        /// rgb = 0,1,2

        let mut has_pallete = false;
        for (index, &(r, g, b)) in palletes.into_iter().enumerate() {
            if pixel.r == r && pixel.g == g && pixel.b == b {
                has_pallete = true;
                content.push(index as u32);
                break;
            } else {
                continue;
            }
        }

        if has_pallete == false {
            println!("image have unexpected color! ({:?}) on coordinate ({}, {}), fallback to \
                      pallete index 0",
                     pixel,
                     x,
                     y);
            content.push(0);
        }

    }

    println!("  image size: {} x {}", width, height);
    (width, height, content)
}

/// check pixel in /r/place randomly,
/// if different with reference image then
/// replace the pixel
/// return true if pixel replaced
/// false othwerwise
fn work(width: u32,
        height: u32,
        pixels: &[u32],
        offset_x: u32,
        offset_y: u32,
        user_token: &UserToken)
        -> bool {
    use rand::distributions::{IndependentSample, Range};

    let between_x = Range::new(0, width);
    let between_y = Range::new(0, height);
    let mut rng = rand::thread_rng();

    let x = between_x.ind_sample(&mut rng);
    let y = between_y.ind_sample(&mut rng);
    let index = (y * width) + x;
    let color = pixels[index as usize];

    let absolute_x = offset_x + x;
    let absolute_y = offset_y + y;
    let is_same = check_pixel(absolute_x, absolute_y, color);
    if is_same == false {
        draw(user_token, absolute_x, absolute_y, color);
        return true;
    } else {
        return false;
    }
}


/// just looping
fn worker_per_user<'a>(width: u32,
                       height: u32,
                       pixels: &'a [u32],
                       offset_x: u32,
                       offset_y: u32,
                       username: &'a str,
                       password: &'a str) {
    const MAX_RETRY: i32 = 5;

    loop {
        let user_token = match login(username, password) {
            Ok(user_token) => user_token,
            Err(why) => {
                println!("cannot login to reddit: {}", why.description());

                /// sleep 1000 ms
                let duration = std::time::Duration::from_millis(1000);
                std::thread::sleep(duration);
                continue;
            }
        };

        let mut is_working = false;
        let mut retry = 0;

        while is_working == false && retry < MAX_RETRY {
            is_working = work(width, height, &pixels, offset_x, offset_y, &user_token);
            retry += 1;

            /// sleep 100 ms
            let duration = std::time::Duration::from_millis(100);
            std::thread::sleep(duration);
        }

        /// wait 5 minutes
        let duration = std::time::Duration::from_millis(1_000 * 60 * 5);
        std::thread::sleep(duration);
    }
}

fn load_toml<T: serde::Deserialize>(config_file: &str) -> Result<T> {
    let path = Path::new(config_file);
    let mut file = std::fs::File::open(&path)?;
    let mut s = String::new();

    file.read_to_string(&mut s)?;
    let ret = toml::from_str::<T>(&s);
    match ret {
        Ok(r) => Ok(r),
        Err(why) => bail!("cannot deserialize toml file: {}", why),
    }
}


/// get user name & password pair from users.toml
fn load_available_accounts() -> Result<Vec<ConfigUser>> {
    let content: ConfigUserToml = load_toml("users.toml")?;
    Ok(content.users)
}

fn main() {
    let config = load_toml("reddit_place.toml");
    let ConfigData { image } = config.unwrap();

    // get users account data
    let users = match load_available_accounts(){
        Ok(users) => users,
        Err(why) => {
            println!("cannot open users.toml: {}", why.description());
            return
        }
    };



    let mut children = vec![];
    for ConfigUser { username, password } in users {
        let image = image.clone();
        children.push(std::thread::spawn(move || {
            println!("thread for user {}", username);

            let offset = image.offset;
            let image_path = &image.path;
            let (width, height, pixels) = load_image(&image_path);
            worker_per_user(width,
                            height,
                            &pixels,
                            offset.x,
                            offset.y,
                            &username,
                            &password);
        }));
    }

    for child in children {
        let _ = child.join();
    }

}
