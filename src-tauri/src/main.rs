#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;

use enigo::{Coordinate, Enigo, Mouse, Settings};
use lazy_static::lazy_static;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tauri::{
    AppHandle, GlobalShortcutManager, GlobalWindowEvent, LogicalPosition, LogicalSize, Manager,
    Monitor, RunEvent, Runtime, Size, SystemTray, SystemTrayEvent,
};
struct MySettings {
    interval: u64,
    should_running: bool,
    last_time_move_mouse: Option<Instant>,
}

lazy_static! {
    static ref MY_SETTINGS: Arc<Mutex<MySettings>> = Arc::new(Mutex::new(MySettings {
        interval: 10,
        should_running: true,
        last_time_move_mouse: None,
    }));
}

fn handle_window_event(event: GlobalWindowEvent<impl Runtime>) {
    match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            let window = event.window();
            if window.label() == "main" {
                info!("minimized and hide instead close window");
                window.hide().unwrap();
                window.minimize().unwrap();
                let mut settings = MY_SETTINGS.lock().unwrap();
                settings.should_running = true;
                api.prevent_close();
            }
        }
        tauri::WindowEvent::Resized(current_size) => {
            let window = event.window();
            if window.label() == "main" {
                info!("size: {}, {}", current_size.width, current_size.height);
                if current_size.width == 0 && current_size.height == 0 {
                    window.hide().unwrap();
                    let mut settings = MY_SETTINGS.lock().unwrap();
                    settings.should_running = true;
                }
            }
        }
        _ => {}
    }
}
fn handle_tray_event(app: &AppHandle<impl Runtime>, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            info!("left click");
            let window = app.get_window("main").unwrap();
            info!("window, current uri: {:?}", window.url());
            let mut settings = MY_SETTINGS.lock().unwrap();

            settings.should_running = !settings.should_running;

            if window.is_visible().unwrap() {
                info!("windows is visible");
                if !window.is_minimized().unwrap() {
                    info!("window is not minimized");
                    window.minimize().unwrap();
                }
                window.hide().unwrap();
            } else {
                info!("windows is not visible");
                if window.is_minimized().unwrap() {
                    info!("window is minimized");
                    window.unminimize().unwrap();
                }
                window.show().unwrap();
            }
        }
        _ => {}
    }
}

fn main() {
    env_logger::init();

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_interval, pause_move,])
        .system_tray(SystemTray::new().with_tooltip("click to show settings or hide"))
        .on_window_event(handle_window_event)
        .on_system_tray_event(handle_tray_event)
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while build tauri application");

    let _ = std::thread::spawn(|| {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        loop {
            let settings = MY_SETTINGS.lock().unwrap();
            let interval = settings.interval;
            let last_time_move_mouse = settings.last_time_move_mouse;
            drop(settings);
            std::thread::sleep(std::time::Duration::from_millis(100));
            match last_time_move_mouse {
                None => {
                    MY_SETTINGS.lock().unwrap().last_time_move_mouse = Some(Instant::now());
                }
                Some(last_time) => {
                    if Instant::now() - last_time > std::time::Duration::from_secs(interval) {
                        MY_SETTINGS.lock().unwrap().last_time_move_mouse = Some(Instant::now());
                        if MY_SETTINGS.lock().unwrap().should_running == false {
                            info!("move mouse pause...");
                            continue;
                        }
                        info!("move mouse");
                        enigo.move_mouse(1, 1, Coordinate::Rel).unwrap();
                    }
                }
            }
        }
    });
    app.run(|app_handle, e| match e {
        // Application is ready (triggered only once)
        RunEvent::Ready => {
            let app_handle = app_handle.clone();
            app_handle
                .global_shortcut_manager()
                .register("Ctrl+Alt+p", move || {
                    info!("find mouse triggered");
                    let enigo = Enigo::new(&Settings::default()).unwrap();
                    let window = app_handle.get_window("mouse_position").unwrap();
                    window.show().unwrap();
                    let mouse_position = enigo.location().unwrap();
                    info!("mouse position: {:?}", mouse_position);
                    let monitors = window.available_monitors().unwrap();
                    let window_size_physical = window.outer_size().unwrap();
                    let mut window_size_logic = LogicalSize {
                        width: window_size_physical.width,
                        height: window_size_physical.height,
                    };
                    for mon in monitors {
                        info!("monitor: {:?}", mon);
                        let real_mon_pos = mon.position().to_logical::<i32>(mon.scale_factor());
                        let real_mon_size = mon.size().to_logical::<i32>(mon.scale_factor());
                        if mouse_position.0 >= real_mon_pos.x
                            && mouse_position.0 <= real_mon_pos.x + real_mon_size.width
                            && mouse_position.1 >= real_mon_pos.y
                            && mouse_position.1 <= real_mon_pos.y + real_mon_size.height
                        {
                            info!("monitor: {:?}", mon);
                            window_size_logic =
                                window_size_physical.to_logical::<u32>(mon.scale_factor());
                            window.set_size(Size::Physical(*mon.size())).unwrap();
                            break;
                        }
                    }
                    info!("window_size_physical: {:?}", window_size_physical);
                    info!("window_size_logic: {:?}", window_size_logic);
                    window
                        .set_position(LogicalPosition {
                            x: mouse_position.0 - (window_size_logic.width as i32 / 2),
                            y: mouse_position.1 - (window_size_logic.height as i32 / 2),
                        })
                        .unwrap();
                    window.emit("find_mouse", {}).unwrap();
                })
                .unwrap();
        }
        _ => {}
    })
}

#[tauri::command]
fn set_interval(interval: u32) {
    info!("set interval: {}", interval);
    let mut settings = MY_SETTINGS.lock().unwrap();
    settings.interval = interval as u64;
    settings.last_time_move_mouse = Some(Instant::now());
}

#[tauri::command]
fn pause_move() {
    info!("pause move");
    let mut settings = MY_SETTINGS.lock().unwrap();
    settings.should_running = false;
}
