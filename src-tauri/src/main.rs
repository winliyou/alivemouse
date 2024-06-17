// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use tauri::{Manager, SystemTray, SystemTrayEvent};
use winapi::um::winuser::{mouse_event, MOUSEEVENTF_MOVE};

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
    let _ = std::thread::spawn(|| loop {
        let settings = MY_SETTINGS.lock().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(settings.interval));
        if settings.should_running == false {
            continue;
        }
        unsafe {
            mouse_event(MOUSEEVENTF_MOVE, 0, 1, 0, 0);
        }
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_interval, pause_move])
        .system_tray(SystemTray::new())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::DoubleClick { .. } => {
                println!("double click");
                let window = app.get_window("main").unwrap();
                let mut settings = MY_SETTINGS.lock().unwrap();

                settings.is_hidden = !settings.is_hidden;
                if settings.is_hidden {
                    window.hide().unwrap();
                    settings.should_running = true;
                } else {
                    window.show().unwrap();
                    settings.should_running = false;
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn set_interval(interval: u32) {
    let mut settings = MY_SETTINGS.lock().unwrap();
    settings.interval = interval as u64;
}

#[tauri::command]
fn pause_move() {
    let mut settings = MY_SETTINGS.lock().unwrap();
    settings.should_running = false;
}
