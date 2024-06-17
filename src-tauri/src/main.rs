#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lazy_static::lazy_static;
use std::{
    mem,
    sync::{Arc, Mutex},
};
use tauri::{Manager, SystemTray, SystemTrayEvent};
use winapi::um::winuser::{SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_MOVE, MOUSEINPUT};

#[derive(serde::Deserialize, serde::Serialize)]
struct MySettings {
    interval: u64,
    should_running: bool,
    is_hidden: bool,
    sss: String,
}

lazy_static! {
    static ref MY_SETTINGS: Arc<Mutex<MySettings>> = Arc::new(Mutex::new(MySettings {
        interval: 10,
        should_running: true,
        is_hidden: true,
        sss: "sss".to_string(),
    }));
}

fn main() {
    let _ = std::thread::spawn(|| loop {
        let settings = MY_SETTINGS.lock().unwrap();
        let interval = settings.interval;
        let should_running = settings.should_running;
        drop(settings);
        if should_running == false {
            std::thread::sleep(std::time::Duration::from_millis(500));
            println!("should_running is false");
            continue;
        }
        println!("move mouse");
        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: unsafe { mem::zeroed() },
        };
        std::thread::sleep(std::time::Duration::from_secs(interval));

        unsafe {
            *input.u.mi_mut() = MOUSEINPUT {
                dx: 1,
                dy: 1,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
        }
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_interval,
            pause_move,
            hide_to_tray
        ])
        .system_tray(SystemTray::new().with_tooltip("click to show settings or hide"))
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                let window = event.window();
                if window.label() == "main" {
                    window.hide().unwrap();
                    let mut settings = MY_SETTINGS.lock().unwrap();
                    settings.is_hidden = true;
                    settings.should_running = true;
                }
            }

            _ => {}
        })
        .on_page_load(|window, _payload| {
            let window_ = window.clone();
            window.listen("hide-to-tray", move |_| {
                window_.hide().unwrap();
            });
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::DoubleClick { .. } => {
                println!("double click");
                let window = app.get_window("main").unwrap();
                println!("window, current uri: {:?}", window.url());
                {
                    let mut settings = MY_SETTINGS.lock().unwrap();

                    settings.is_hidden = !settings.is_hidden;
                    if settings.is_hidden {
                        window.hide().unwrap();
                        settings.should_running = true;
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        settings.should_running = false;
                    }
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn set_interval(interval: u32) {
    println!("set interval: {}", interval);
    {
        let mut settings = MY_SETTINGS.lock().unwrap();
        settings.interval = interval as u64;
    }
}

#[tauri::command]
fn pause_move() {
    println!("pause move");
    {
        let mut settings = MY_SETTINGS.lock().unwrap();
        settings.should_running = false;
    }
}

#[tauri::command]
fn hide_to_tray(window: tauri::Window) {
    println!("hide to tray");
    {
        let mut settings = MY_SETTINGS.lock().unwrap();
        settings.is_hidden = true;
        settings.should_running = true;
    }
    window.emit("hide-window", ()).unwrap();
}
