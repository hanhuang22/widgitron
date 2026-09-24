use once_cell::sync::OnceCell;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::config_store;
use crate::models::AppConfig;

const SIDEBAR_LABEL: &str = "sidebar";
const DEFAULT_THICKNESS_LOGICAL: f64 = 320.0;
pub const DEFAULT_REVEAL_SENSITIVITY: u8 = 4;
pub const DEFAULT_HIDE_SENSITIVITY: u8 = 8;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SidebarDockState {
    pub edge: String,
    pub pinned: bool,
    pub expanded: bool,
    pub dragging: bool,
    pub preview_edge: Option<String>,
}

struct DockRuntime {
    state: SidebarDockState,
    thickness: f64,
    length: Option<f64>,
    monitor_anchor: Option<(i32, i32)>,
}

impl DockRuntime {
    fn from_config(config: &AppConfig) -> Self {
        Self {
            state: SidebarDockState {
                edge: parse_edge(config.sidebar_edge.as_deref()).into(),
                pinned: config.sidebar_pinned.unwrap_or(false),
                expanded: config.sidebar_pinned.unwrap_or(false),
                dragging: false,
                preview_edge: None,
            },
            thickness: config.sidebar_width.unwrap_or(DEFAULT_THICKNESS_LOGICAL),
            length: config.sidebar_length,
            monitor_anchor: config.sidebar_monitor_x.zip(config.sidebar_monitor_y),
        }
    }
}

static DOCK_RUNTIME: OnceCell<Mutex<DockRuntime>> = OnceCell::new();

fn parse_edge(value: Option<&str>) -> &'static str {
    match value {
        Some("left") => "left",
        Some("top") => "top",
        Some("bottom") => "bottom",
        _ => "right",
    }
}

fn lock_runtime() -> Result<std::sync::MutexGuard<'static, DockRuntime>, String> {
    DOCK_RUNTIME
        .get()
        .ok_or_else(|| "Sidebar dock controller is unavailable".to_string())?
        .lock()
        .map_err(|_| "Sidebar dock controller lock is poisoned".to_string())
}

pub fn ensure_sidebar_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(SIDEBAR_LABEL) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, SIDEBAR_LABEL, WebviewUrl::App("index.html".into()))
        .title("Widgitron Sidebar")
        .inner_size(DEFAULT_THICKNESS_LOGICAL, 900.0)
        .decorations(false)
        .resizable(true)
        .maximizable(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|err| err.to_string())
}

pub fn start(app: AppHandle, config: &AppConfig) -> Result<(), String> {
    if DOCK_RUNTIME.get().is_some() {
        apply_config(&app, config);
        return Ok(());
    }

    let window = ensure_sidebar_window(&app)?;
    crate::ui_scale::apply_to_window(&window, crate::ui_scale::from_config(config))?;
    let runtime = DockRuntime::from_config(config);
    let expanded = runtime.state.expanded;
    DOCK_RUNTIME
        .set(Mutex::new(runtime))
        .map_err(|_| "Sidebar dock controller is already running".to_string())?;

    let blur_app = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Focused(false)) {
            let should_collapse = lock_runtime()
                .map(|runtime| runtime.state.expanded && !runtime.state.pinned)
                .unwrap_or(false);
            if should_collapse {
                let _ = collapse(&blur_app);
            }
        }
    });

    position_sidebar(&app, &window)?;
    if expanded {
        window.show().map_err(|err| err.to_string())?;
    }
    emit_state(&app);
    Ok(())
}

pub fn apply_config(app: &AppHandle, config: &AppConfig) {
    let Ok(mut runtime) = lock_runtime() else {
        return;
    };
    let expanded = runtime.state.expanded;
    *runtime = DockRuntime::from_config(config);
    runtime.state.expanded = runtime.state.pinned || expanded;
    let expanded = runtime.state.expanded;
    drop(runtime);

    if let Ok(window) = ensure_sidebar_window(app) {
        if let Err(err) =
            crate::ui_scale::apply_to_window(&window, crate::ui_scale::from_config(config))
        {
            log::warn!("Failed to apply UI scale to sidebar: {err}");
        }
        if let Err(err) = position_sidebar(app, &window) {
            log::warn!("Failed to position sidebar: {err}");
        }
        let visibility_result = if expanded {
            window.show()
        } else {
            window.hide()
        };
        if let Err(err) = visibility_result {
            log::warn!("Failed to update sidebar visibility: {err}");
        }
    }
    emit_state(app);
}

pub fn show(app: &AppHandle, focus: bool) -> Result<(), String> {
    ensure_started(app)?;
    let window = ensure_sidebar_window(app)?;
    position_sidebar(app, &window)?;
    window.show().map_err(|err| err.to_string())?;
    if focus {
        window.set_focus().map_err(|err| err.to_string())?;
    }
    lock_runtime()?.state.expanded = true;
    emit_state(app);
    Ok(())
}

pub fn collapse(app: &AppHandle) -> Result<(), String> {
    ensure_started(app)?;
    {
        let mut runtime = lock_runtime()?;
        runtime.state.expanded = false;
        runtime.state.pinned = false;
    }
    if let Some(window) = app.get_webview_window(SIDEBAR_LABEL) {
        window.hide().map_err(|err| err.to_string())?;
    }
    persist_pinned(app);
    emit_state(app);
    Ok(())
}

pub fn set_pinned(app: &AppHandle, pinned: bool, focus: bool) -> Result<SidebarDockState, String> {
    ensure_started(app)?;
    {
        let mut runtime = lock_runtime()?;
        runtime.state.pinned = pinned;
    }
    if pinned {
        show(app, focus)?;
    } else {
        emit_state(app);
    }
    persist_pinned(app);
    get_state(app)
}

pub fn toggle_pinned(
    app: &AppHandle,
    focus_when_pinning: bool,
) -> Result<SidebarDockState, String> {
    ensure_started(app)?;
    let pinned = !lock_runtime()?.state.pinned;
    set_pinned(app, pinned, focus_when_pinning && pinned)
}

pub fn begin_drag(app: &AppHandle) -> Result<SidebarDockState, String> {
    ensure_started(app)?;
    ensure_sidebar_window(app)?
        .start_dragging()
        .map_err(|err| err.to_string())?;
    get_state(app)
}

pub fn get_state(app: &AppHandle) -> Result<SidebarDockState, String> {
    ensure_started(app)?;
    Ok(lock_runtime()?.state.clone())
}

fn ensure_started(app: &AppHandle) -> Result<(), String> {
    if DOCK_RUNTIME.get().is_none() {
        let config = config_store::read_config::<AppConfig>(app, "app_config.json");
        start(app.clone(), &config)?;
    }
    Ok(())
}

fn emit_state(app: &AppHandle) {
    if let Ok(runtime) = lock_runtime() {
        let _ = app.emit("sidebar_state_update", runtime.state.clone());
    }
}

fn persist_pinned(app: &AppHandle) {
    let Ok(runtime) = lock_runtime() else {
        return;
    };
    let pinned = runtime.state.pinned;
    drop(runtime);
    let mut config = config_store::read_config::<AppConfig>(app, "app_config.json");
    config.sidebar_pinned = Some(pinned);
    if let Err(err) = config_store::write_config(app, "app_config.json", &config) {
        log::warn!("Failed to persist sidebar pin state: {err}");
    }
}

fn sidebar_monitor(app: &AppHandle, anchor: Option<(i32, i32)>) -> Result<Monitor, String> {
    let point = anchor.or_else(|| {
        app.cursor_position()
            .ok()
            .map(|position| (position.x as i32, position.y as i32))
    });
    if let Some((x, y)) = point {
        if let Ok(Some(monitor)) = app.monitor_from_point(x as f64, y as f64) {
            return Ok(monitor);
        }
    }
    app.primary_monitor()
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "No monitor available for sidebar docking".to_string())
}

fn position_sidebar(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let runtime = lock_runtime()?;
    let edge = runtime.state.edge.clone();
    let thickness = runtime.thickness.max(DEFAULT_THICKNESS_LOGICAL);
    let length = runtime.length;
    let anchor = runtime.monitor_anchor;
    drop(runtime);

    let monitor = sidebar_monitor(app, anchor)?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let vertical = edge == "left" || edge == "right";
    let cross_size = if vertical {
        area.size.width
    } else {
        area.size.height
    };
    let edge_size = if vertical {
        area.size.height
    } else {
        area.size.width
    };
    let thickness_px = ((thickness * scale).round() as u32).clamp(1, cross_size);
    let length_px = length
        .map(|value| (value * scale).round() as u32)
        .unwrap_or(edge_size)
        .clamp(1, edge_size);
    let (width, height) = if vertical {
        (thickness_px, length_px)
    } else {
        (length_px, thickness_px)
    };
    let x = match edge.as_str() {
        "left" => area.position.x,
        "right" => area.position.x + area.size.width as i32 - width as i32,
        _ => area.position.x + (area.size.width as i32 - width as i32) / 2,
    };
    let y = match edge.as_str() {
        "top" => area.position.y,
        "bottom" => area.position.y + area.size.height as i32 - height as i32,
        _ => area.position.y + (area.size.height as i32 - height as i32) / 2,
    };
    window
        .set_size(PhysicalSize::new(width, height))
        .map_err(|err| err.to_string())?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|err| err.to_string())
}
