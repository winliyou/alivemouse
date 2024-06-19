#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::info;

use enigo::{Coordinate, Enigo, Mouse, Settings};
use lazy_static::lazy_static;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tauri::{
    AppHandle, CustomMenuItem, GlobalShortcutManager, GlobalWindowEvent, LogicalPosition,
    LogicalSize, Manager, RunEvent, Runtime, SystemTray, SystemTrayEvent, SystemTrayMenu,
};
struct MySettings {
    interval: u64,
    should_running: bool,
    last_time_move_mouse: Option<Instant>,
    hotkey: String,
}

lazy_static! {
    static ref MY_SETTINGS: Arc<Mutex<MySettings>> = Arc::new(Mutex::new(MySettings {
        interval: 10,
        should_running: true,
        last_time_move_mouse: None,
        hotkey: "Ctrl+Alt+P".to_string(),
    }));
}

#[derive(serde::Deserialize, serde::Serialize)]
struct MoveMouseIntervalChangePayload {
    interval: u64,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct FindMouseHotKeyChangePayload {
    hotkey: String,
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
            let tray_handle = app.tray_handle();
            tray_handle.set_menu(SystemTrayMenu::new()).unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "Quit" => {
                app.exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

fn setup_find_mouse_stuff(app_handle: AppHandle) {
    let _app_handle_1 = app_handle.clone();
    app_handle
        .global_shortcut_manager()
        .register(&MY_SETTINGS.lock().unwrap().hotkey, move || {
            handle_global_shortcut_find_mouse(&_app_handle_1);
        })
        .unwrap();
    let setting_window = app_handle.get_window("main").unwrap();
    setting_window
        .clone()
        .listen("find_mouse_hotkey_change", move |event| {
            let find_mouse_hotkey_change_payload: FindMouseHotKeyChangePayload =
                serde_json::from_str(event.payload().unwrap()).unwrap();
            let _app_handle = app_handle.clone();
            match _app_handle
                .global_shortcut_manager()
                .unregister(&MY_SETTINGS.lock().unwrap().hotkey)
            {
                Ok(_) => {}
                Err(e) => {
                    tauri::api::dialog::message(
                        Some(&setting_window.clone()),
                        "不能解注册这个快捷键",
                        format!(
                            "解注册快捷键({})失败: {}",
                            &MY_SETTINGS.lock().unwrap().hotkey,
                            e
                        ),
                    );
                }
            }
            match _app_handle.global_shortcut_manager().register(
                &find_mouse_hotkey_change_payload.hotkey,
                move || {
                    handle_global_shortcut_find_mouse(&_app_handle);
                },
            ) {
                Ok(_) => {}
                Err(e) => {
                    tauri::api::dialog::message(
                        Some(&setting_window.clone()),
                        "不能注册这个快捷键",
                        format!(
                            "注册快捷键({})失败: {}",
                            find_mouse_hotkey_change_payload.hotkey, e
                        ),
                    );
                }
            }

            MY_SETTINGS.lock().unwrap().hotkey = find_mouse_hotkey_change_payload.hotkey.clone();
        });
}

fn handle_global_shortcut_find_mouse(_app_handle: &AppHandle) {
    info!("find mouse triggered");
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let window = _app_handle.get_window("mouse_position").unwrap();
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
            window_size_logic = window_size_physical.to_logical::<u32>(mon.scale_factor());
            window.set_size(window_size_logic).unwrap();
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
    info!("window current monitor: {:?}", window.current_monitor());
    window.emit("find_mouse", {}).unwrap();
}

fn main() {
    env_logger::init();

    let app = tauri::Builder::default()
        .system_tray(
            SystemTray::new()
                .with_tooltip("click to show settings or hide")
                .with_menu(SystemTrayMenu::new().add_item(CustomMenuItem::new("Quit", "Quit App"))),
        )
        .on_window_event(handle_window_event)
        .on_system_tray_event(handle_tray_event)
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            _app.set_activation_policy(tauri::ActivationPolicy::Accessory);
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
            let app_handle_1 = app_handle.clone();
            setup_find_mouse_stuff(app_handle_1);
            let setting_window = app_handle.get_window("main").unwrap();
            setting_window.listen("move_mouse_interval_change", |event| {
                let move_mouse_interval_change_payload: MoveMouseIntervalChangePayload =
                    serde_json::from_str(event.payload().unwrap()).unwrap();
                MY_SETTINGS.lock().unwrap().interval = move_mouse_interval_change_payload.interval;
            });
        }
        _ => {}
    })
}
