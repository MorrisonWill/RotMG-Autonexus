extern crate serde;
extern crate toml;
extern crate winapi;

use toml::Value;

use serde::Serialize;

use self::winapi::ctypes::c_int;
use self::winapi::um::winuser::*;

use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::iter::once;
use std::mem::size_of;
use std::mem::transmute_copy;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::thread;
use std::time;

#[derive(Serialize, Default)]
struct Config {
    pixel_x: i32,
    pixel_y: i32,
    debug: bool,
    delay: u64,
}

fn nexus() {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe {
            transmute_copy(&KEYBDINPUT {
                wVk: 0x52, // keycode for r
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    unsafe { SendInput(1, &mut input as LPINPUT, size_of::<INPUT>() as c_int) };
}

fn debug(x: i32, y: i32, delay: u64) {
    let hdc_source = unsafe { GetDC(ptr::null_mut()) };
    loop {
        let pixel = unsafe { winapi::um::wingdi::GetPixel(hdc_source, x, y) };
        println!("The color at ({0}, {1}) is {2}.", x, y, pixel);
        thread::sleep(time::Duration::from_millis(delay))
    }
}

fn autonexus_loop(x: i32, y: i32, delay: u64) {
    let window: Vec<u16> = OsStr::new("RotMGExalt")
        .encode_wide()
        .chain(once(0))
        .collect();

    let window_handle = unsafe { FindWindowW(ptr::null_mut(), window.as_ptr()) };
    let hdc_source = unsafe { GetDC(ptr::null_mut()) };

    println!("Monitoring health bar.");

    loop {
        let foreground = unsafe { GetForegroundWindow() };
        if foreground == window_handle {
            let pixel = unsafe { winapi::um::wingdi::GetPixel(hdc_source, x, y) };
            if pixel > 10000000 {
                println!("Nexused!");
                nexus();
            }
        }

        thread::sleep(time::Duration::from_millis(delay))
    }
}

fn main() {
    let config_exists = std::path::Path::new("config.toml").exists();

    if !config_exists {
        let _new_file = File::create("config.toml").unwrap();

        let data = Config::default();

        let toml_string = toml::to_string(&data).expect("Could not encode TOML value.");

        fs::write("config.toml", toml_string).expect("Could not write to config file.");

        println!("Config file was not found so a blank one has been created.");

        return;
    }

    let mut config = File::open("config.toml").unwrap();

    let mut config_content = String::new();
    let _bytes_read = config.read_to_string(&mut config_content).unwrap();

    let config_data = config_content.parse::<Value>().unwrap();

    let x: i32 = config_data["pixel_x"]
        .as_integer()
        .unwrap()
        .try_into()
        .unwrap();
    let y: i32 = config_data["pixel_y"]
        .as_integer()
        .unwrap()
        .try_into()
        .unwrap();
    let delay: u64 = config_data["pixel_y"]
        .as_integer()
        .unwrap()
        .try_into()
        .unwrap();

    if config_data["debug"].as_bool().unwrap() {
        println!("Starting to debug.");
        debug(x, y, delay);
    }

    autonexus_loop(x, y, delay);
}
