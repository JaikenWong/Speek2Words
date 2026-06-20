use tauri::Manager;

mod asr;
mod input;
mod recorder;
mod store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let settings_item = tauri::menu::MenuItem::new(app, "Settings", true, None::<&str>)?;
            let quit_item = tauri::menu::MenuItem::new(app, "Quit", true, None::<&str>)?;
            let menu = tauri::menu::MenuBuilder::new(app)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("Speek2Words")
                .menu(&menu)
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|tray, event| {
                    match event.id.as_ref() {
                        "Settings" => {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "Quit" => {
                            tray.app_handle().exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            tray.set_visible(true)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            recorder::start_recording,
            recorder::stop_and_transcribe,
            input::type_text,
            store::get_config,
            store::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
