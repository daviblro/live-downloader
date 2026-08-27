use std::sync::atomic::Ordering;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::{app::AppState, platform::shell::open_directory};

pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("application icon".into()))?;
    let show = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause all recordings", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Resume monitoring", true, None::<&str>)?;
    let downloads = MenuItem::with_id(app, "downloads", "Open downloads", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Exit Live Downloader", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[&show, &pause, &resume, &downloads, &separator, &quit],
    )?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("Live Downloader")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "pause" => {
                let engine = app.state::<AppState>().engine.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = engine.pause_all().await;
                });
            }
            "resume" => {
                let _ = app.state::<AppState>().engine.start();
            }
            "downloads" => {
                let database = app.state::<AppState>().database.clone();
                tauri::async_runtime::spawn(async move {
                    if let Ok(settings) = database.settings() {
                        let _ = open_directory(std::path::Path::new(&settings.download_directory));
                    }
                });
            }
            "quit" => {
                app.state::<AppState>()
                    .is_quitting
                    .store(true, Ordering::Relaxed);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
