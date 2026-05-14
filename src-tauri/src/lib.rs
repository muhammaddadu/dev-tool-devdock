pub mod app_state;
pub mod commands;
pub mod core;
pub mod logging;
pub mod models;
pub mod platform;
pub mod storage;
pub mod tray;
pub mod utils;

use app_state::AppState;
use commands::{ai, ports, projects, services, settings};
use tauri::Manager;
use utils::paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,devdock_lib=debug")),
        )
        .init();

    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // macOS LaunchAgent. The plugin writes ~/Library/LaunchAgents/...plist
        // when enabled, deletes it when disabled. No explicit args needed —
        // we launch the app fresh and the tray-only setup hook hides the window.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // Auto-update infrastructure. Inert until `endpoints` + `pubkey` are
        // configured in tauri.conf.json — see docs/engineering/release.md.
        // `tauri-plugin-process` is required so the updater can relaunch the
        // app after install.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // Native folder picker for the Add Command dialog.
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(|app| {
            // macOS: run as a menu-bar-only app. Force the NSWindow background
            // fully clear at the AppKit layer so the rounded panel CSS cuts
            // through to the desktop with no rectangular halo. We are not
            // attempting NSVisualEffectView vibrancy here — it fought CSS at
            // every turn on this machine; a clean solid rounded popover is
            // better than a half-working translucent one.
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                if let Some(window) = app.get_webview_window("main") {
                    unsafe {
                        force_clear_window_background(&window);
                    }
                }
            }

            // Open SQLite synchronously so commands never fire against an
            // unmanaged pool. Setup is the right place — async work spawned
            // later races with the very first refresh.
            let db_path = paths::app_data_dir().join("devdock.sqlite");
            let pool = tauri::async_runtime::block_on(storage::db::open(&db_path))?;
            tracing::info!(path = %db_path.display(), "opened devdock.sqlite");
            app.manage(pool);

            tray::install_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            services::list_services,
            services::refresh_services,
            services::save_detected_service,
            services::update_saved_service,
            services::list_saved_services,
            services::unsave_service,
            services::set_service_pinned,
            services::run_service,
            services::stop_service,
            services::restart_service,
            services::open_url,
            services::open_path,
            services::open_in_terminal,
            services::list_editors,
            services::open_in_editor,
            services::read_log_tail,
            services::quit_app,
            ports::kill_detected_process,
            projects::scan_project,
            settings::get_setting,
            settings::set_setting,
            settings::list_settings,
            ai::detect_ai_providers,
            ai::analyze_project_with_ai,
        ])
        .on_window_event(|window, event| match event {
            // Close button (or Cmd-W) should hide the panel, not quit the app.
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            // Menu-bar app convention: clicking outside the panel hides it.
            // Tauri emits Focused(false) when the user clicks anywhere else.
            tauri::WindowEvent::Focused(false) => {
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running DevDock");
}

/// AppKit fallback used to guarantee the NSWindow itself is fully clear.
/// Tauri's `transparent: true` only sets this on initial create on some
/// macOS builds; if anything ever flips the window back to opaque (focus
/// change, fullscreen transitions, etc.) the rounded popover illusion
/// breaks. Calling this in `setup` makes the behaviour deterministic.
#[cfg(target_os = "macos")]
unsafe fn force_clear_window_background(window: &tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    let Ok(ns_window_raw) = window.ns_window() else {
        return;
    };
    let ns_window = ns_window_raw as *mut AnyObject;
    if ns_window.is_null() {
        return;
    }
    let ns_color_cls = objc2::class!(NSColor);
    let clear_color: *mut AnyObject = msg_send![ns_color_cls, clearColor];
    let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
    let _: () = msg_send![ns_window, setOpaque: false];
    let _: () = msg_send![ns_window, setHasShadow: false];
}
