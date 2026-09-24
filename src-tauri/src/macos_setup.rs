use tauri::AppHandle;

use crate::config_store;
use crate::models::AppConfig;

const WIDGETS: [(&str, &str); 4] = [
    ("widget-gpu-default", "theme-gpu-default"),
    ("widget-deadlines-default", "theme-deadline-default"),
    ("widget-arxiv-default", "theme-arxiv-default"),
    ("widget-quota-default", "theme-quota-default"),
];

pub fn apply_first_run_defaults(app: &AppHandle) -> Result<(), String> {
    let mut config = config_store::read_config::<AppConfig>(app, "app_config.json");
    if config.macos_setup_version.unwrap_or_default() >= 1 {
        return Ok(());
    }

    let mut active = config.active_widgets.take().unwrap_or_default();
    let legacy_auto_opened_all = WIDGETS
        .iter()
        .all(|(widget, _)| active.get(*widget) == Some(&true));
    for (widget, _) in WIDGETS {
        if legacy_auto_opened_all {
            let explicitly_pinned = config
                .always_on_top
                .as_ref()
                .and_then(|values| values.get(widget))
                .copied()
                .unwrap_or(false);
            active.insert(widget.to_string(), explicitly_pinned);
        } else {
            active.entry(widget.to_string()).or_insert(false);
        }
    }
    config.active_widgets = Some(active);

    if config.language.is_none() {
        config.language = Some("zh-CN".into());
    }
    if let Some(theme) = config.sidebar_theme.as_mut() {
        if theme.background_opacity == Some(0.84) {
            theme.background_opacity = Some(0.96);
        }
        if theme.header_opacity == Some(0.9) {
            theme.header_opacity = Some(0.98);
        }
        if theme.card_opacity == Some(0.76) {
            theme.card_opacity = Some(0.94);
        }
    }

    let mut themes = config_store::read_theme_config(app);
    for (widget, opaque_theme) in WIDGETS {
        if themes
            .assignments
            .get(widget)
            .is_some_and(|theme| theme.ends_with("-transparent"))
        {
            themes
                .assignments
                .insert(widget.to_string(), opaque_theme.to_string());
        }
    }
    config_store::write_theme_config(app, &themes)?;
    config.macos_setup_version = Some(1);
    config_store::write_config(app, "app_config.json", &config)
}
