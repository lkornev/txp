use tokio::time::{sleep, Duration};
use lazy_static::lazy_static;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{thread, time};
use std::collections::VecDeque;
use reqwest;
use regex::Regex;
use enigo::*;

// TODO take from the environment vatiable
const AUTH_TOKEN: &str = "Basic bHNrb3JuZXY6MzI1NDMyNTQ=";
// TODO take from the environment vatiable
const RING_BUFFER_SIZE: usize = 15;
const WAIT_TEXT: &str = "..wait";
const ACTION_WORD: &str = "t";
const ERROR_TEXT: &str = "Error!";

lazy_static! {
    // TODO take from the environment vatiable
    static ref LISTENING_PATTERN: Regex = Regex::new(r"(CMI|NP|AG|PM|GEO)-([0-9]+) (T)$")
        .unwrap();
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    println!("Listening for the pattern... ");

    let client = reqwest::Client::new();

    loop {
        let task_name = wait_for_typing().await;
        // TODO take from the environment vatiable
        let url = format!("https://jira.int.tsum.com/browse/{}", task_name);

        let (title, _) = tokio::join!(
            get_page_title(&client, &url),
            print_text(WAIT_TEXT),
        );

        send_backspaces(action_text_len()).await;

        if let Ok(title) = title {
            print_text(&title).await;
        } else {
            print_text(ERROR_TEXT).await;
            send_backspaces(error_text_len()).await;
        }
    }
}

async fn get_page_title(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let h1_re = Regex::new(r#"<h1.*?id="summary-val".*?>(.*?)</h1>"#).unwrap();

    let res = get_page(&client, &url).await;

    match res {
        Ok(res) => {
            match h1_re.captures(&res) {
                Some(cap) => {
                    let title = cap[1].trim().replace("&quot;", "\"");

                    Ok(title)
                },
                None => Err(String::from("Page title not found")),
            }
        },
        Err(_) => Err(String::from("Page cannot be fetched")),
    }
}

async fn get_page(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    client.get(url)
        .header("Authorization", AUTH_TOKEN)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .text()
        .await
}

async fn wait_for_typing() -> String {
    let device_state = DeviceState::new();
    let mut prev_keys: Vec<Keycode> = vec![];
    let mut ring_buffer: VecDeque<char> = VecDeque::new();

    let delay = time::Duration::from_millis(50);

    loop {
        let keys = device_state.get_keys();

        for key in &keys {
            if !prev_keys.contains(&key) {
                match key {
                    Keycode::Backspace => {
                        let ring_len = ring_buffer.len();
                        if ring_len > 2 {
                            ring_buffer.drain(ring_len - 1..ring_len);
                        }
                    },
                    key => {
                        if let Some(letter) = key.to_text() {
                            ring_buffer.push_back(letter);
        
                            if ring_buffer.len() > RING_BUFFER_SIZE {
                                ring_buffer.pop_front();
                            }
            
                            let text = ring_buffer.iter().collect::<String>();
            
                            if let Ok(task_name) = ends_with_pattern(&text) {
                                return task_name;
                            }
                        }
                    }
                }
            }
        }

        prev_keys = keys;

        thread::sleep(delay);
    }
}

fn ends_with_pattern(string: &str) -> Result<String, &str> {
    match LISTENING_PATTERN.captures(string) {
        Some(cap) => {
            Ok(format!("{}-{}", &cap[1], &cap[2]))
        },
        None => Err("Not matched"),
    }
}

async fn print_text(text: &str) {
    let mut enigo = Enigo::new();
    sleep(Duration::from_millis(100)).await;

    for char in text.chars() {
        let letter = char.to_string();
        enigo.key_sequence(&letter);
        sleep(Duration::from_millis(50)).await;
    }
}

async fn send_backspaces(count: usize) {
    sleep(Duration::from_millis(300)).await;

    let mut enigo = Enigo::new();

    for _ in 0..count {
        enigo.key_down(Key::Backspace);
        enigo.key_up(Key::Backspace);
    } 
}

fn action_text_len() -> usize {
    ACTION_WORD.len() + WAIT_TEXT.len()
}

fn error_text_len() -> usize {
    ERROR_TEXT.len()
}

trait Textual {
    fn to_text(&self) -> Option<char>;
}

impl Textual for device_query::Keycode {
    fn to_text(&self) -> Option<char> {
        match self {
            Keycode::Key0 => Some('0'),
            Keycode::Key1 => Some('1'),
            Keycode::Key2 => Some('2'),
            Keycode::Key3 => Some('3'),
            Keycode::Key4 => Some('4'),
            Keycode::Key5 => Some('5'),
            Keycode::Key6 => Some('6'),
            Keycode::Key7 => Some('7'),
            Keycode::Key8 => Some('8'),
            Keycode::Key9 => Some('9'),
            Keycode::A => Some('A'),
            Keycode::B => Some('B'),
            Keycode::C => Some('C'),
            Keycode::D => Some('D'),
            Keycode::E => Some('E'),
            Keycode::F => Some('F'),
            Keycode::G => Some('G'),
            Keycode::H => Some('H'),
            Keycode::I => Some('I'),
            Keycode::J => Some('J'),
            Keycode::K => Some('K'),
            Keycode::L => Some('L'),
            Keycode::M => Some('M'),
            Keycode::N => Some('N'),
            Keycode::O => Some('O'),
            Keycode::P => Some('P'),
            Keycode::Q => Some('Q'),
            Keycode::R => Some('R'),
            Keycode::S => Some('S'),
            Keycode::T => Some('T'),
            Keycode::U => Some('U'),
            Keycode::V => Some('V'),
            Keycode::W => Some('W'),
            Keycode::X => Some('X'),
            Keycode::Y => Some('Y'),
            Keycode::Z => Some('Z'),
            Keycode::Space => Some(' '),
            Keycode::Minus => Some('-'),
            _ => None
        }
    }
}
