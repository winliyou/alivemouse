#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use enigo::{Coordinate, Enigo, Mouse, Settings};
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use tauri::{Manager, SystemTray, SystemTrayEvent};
#[derive(serde::Deserialize, serde::Serialize)]
struct MySettings {
    interval: u64,
    should_running: bool,
    is_hidden: bool,
}

lazy_static! {
    static ref MY_SETTINGS: Arc<Mutex<MySettings>> = Arc::new(Mutex::new(MySettings {
        interval: 10,
        should_running: true,
        is_hidden: true,
    }));
}

fn main() {
    let _ = std::thread::spawn(|| {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        loop {
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
            enigo.move_mouse(1, 1, Coordinate::Rel).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(interval));
        }
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_interval, pause_move,])
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
            tauri::WindowEvent::Resized(current_size) => {
                let window = event.window();
                if window.label() == "main" {
                    println!("size: {}, {}", current_size.width, current_size.height);
                    if current_size.width == 0 && current_size.height == 0 {
                        window.hide().unwrap();
                        let mut settings = MY_SETTINGS.lock().unwrap();
                        settings.is_hidden = true;
                        settings.should_running = true;
                    }
                }
            }
            _ => {}
        })
        .on_page_load(|window, _payload| {
            let window_ = window.clone();
            window.listen("hide_to_tray", move |_| {
                println!("hide to tray");
                {
                    let mut settings = MY_SETTINGS.lock().unwrap();
                    settings.is_hidden = true;
                    settings.should_running = true;
                }
                window_.hide().unwrap();
            });
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::DoubleClick { .. } => {
                println!("double click");
                let window = app.get_window("main").unwrap();
                println!("window, current uri: {:?}", window.url());
                let mut settings = MY_SETTINGS.lock().unwrap();

                settings.is_hidden = !settings.is_hidden;
                if settings.is_hidden {
                    window.hide().unwrap();
                    settings.should_running = true;
                } else {
                    settings.should_running = false;
                    if window.is_minimized().unwrap() {
                        window.unminimize().unwrap();
                    }
                    window.show().unwrap();
                    window.set_focus().unwrap();
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
    let mut settings = MY_SETTINGS.lock().unwrap();
    settings.interval = interval as u64;
}

#[tauri::command]
fn pause_move() {
    println!("pause move");
    let mut settings = MY_SETTINGS.lock().unwrap();
    settings.should_running = false;
}
