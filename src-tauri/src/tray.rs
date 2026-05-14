//! Menu bar tray icon + panel positioning.
//!
//! On macOS the app runs with `ActivationPolicy::Accessory` (no Dock icon).
//! Clicking the tray icon toggles the panel; it appears anchored just below
//! the icon's rect on the active monitor.

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Position, Size, WebviewWindow,
};

use crate::app_state::AppState;

const PANEL_LABEL: &str = "main";
const PANEL_GAP_PX: f64 = 6.0;
const TRAY_ID: &str = "devdock-tray";

/// Template-style menu bar icon (alpha-only black on transparent). Embedded so
/// the tray works regardless of the app's resource resolution at runtime.
const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray.png");

pub fn install_tray(app: &AppHandle) -> tauri::Result<()> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit DevDock", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Show Panel", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let icon = Image::from_bytes(TRAY_ICON_BYTES)?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .tooltip("DevDock")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => request_quit(app),
            "show" => {
                if let Some(window) = app.get_webview_window(PANEL_LABEL) {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window(PANEL_LABEL) {
                    toggle_panel(&window, rect.position, rect.size);
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Handle the tray's Quit menu item. If DevDock isn't managing anything,
/// exit immediately. Otherwise show the panel and emit a `confirm-quit`
/// event so the frontend can prompt the user — sending SIGTERM to managed
/// children before exit prevents orphan processes from outliving the app.
fn request_quit(app: &AppHandle) {
    let count = app
        .try_state::<AppState>()
        .map(|s| s.runtime.snapshot().len())
        .unwrap_or(0);

    if count == 0 {
        app.exit(0);
        return;
    }

    if let Some(window) = app.get_webview_window(PANEL_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
    if let Err(err) = app.emit("devdock://confirm-quit", count) {
        tracing::warn!(error = %err, "failed to emit confirm-quit; exiting anyway");
        app.exit(0);
    }
}

/// Update the small text shown next to the tray icon (macOS menu bar
/// convention). We use it to surface the count of foreground "Dev" services
/// so the user can see at a glance whether anything is running, even when
/// the panel is closed. Falsey counts hide the title entirely.
pub fn set_dev_badge(app: &AppHandle, count: usize) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let title = if count == 0 {
            None
        } else {
            Some(format!("{count}"))
        };
        let _ = tray.set_title(title);
    }
}

fn toggle_panel(window: &WebviewWindow, tray_position: Position, tray_size: Size) {
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
        return;
    }

    if let Some(pos) = compute_panel_position(window, tray_position, tray_size) {
        let _ = window.set_position(pos);
    }
    let _ = window.show();
    let _ = window.set_focus();
}

fn compute_panel_position(
    window: &WebviewWindow,
    tray_position: Position,
    tray_size: Size,
) -> Option<LogicalPosition<f64>> {
    let scale = window.scale_factor().unwrap_or(1.0);

    let tray_pos: LogicalPosition<f64> = tray_position.to_logical(scale);
    let tray_size: LogicalSize<f64> = tray_size.to_logical(scale);

    let panel_size_phys = window.outer_size().ok()?;
    let panel_width = panel_size_phys.width as f64 / scale;

    let tray_center_x = tray_pos.x + tray_size.width / 2.0;
    let mut x = tray_center_x - panel_width / 2.0;
    let y = tray_pos.y + tray_size.height + PANEL_GAP_PX;

    if let Ok(Some(monitor)) = window.current_monitor() {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let m_logical_x = m_pos.x as f64 / scale;
        let m_logical_w = m_size.width as f64 / scale;
        let min_x = m_logical_x + 6.0;
        let max_x = m_logical_x + m_logical_w - panel_width - 6.0;
        x = x.clamp(min_x, max_x.max(min_x));
    }

    Some(LogicalPosition::new(x, y))
}
