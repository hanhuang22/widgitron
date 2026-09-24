//! The WidgetKit extension reads this display-only snapshot from our shared container.
//! Never put credentials or raw provider responses in this file.

use serde::Serialize;
use std::fs;
use tauri::{AppHandle, Manager};

use crate::models::{QuotaConfig, QuotaItem};

const GROUP_ID: &str = "group.com.evan.widgitron";
const SNAPSHOT_FILE: &str = "quota-snapshot.json";

#[derive(Serialize)]
struct QuotaSnapshot<'a> {
    language: &'a str,
    show_plan_type: bool,
    items: Vec<QuotaSnapshotItem<'a>>,
}

#[derive(Serialize)]
struct QuotaSnapshotItem<'a> {
    name: &'a str,
    provider: &'a str,
    current_value: Option<f64>,
    max_quota: Option<f64>,
    unit: Option<&'a str>,
    primary_name: Option<&'a str>,
    primary_reset: Option<&'a str>,
    last_update: Option<&'a str>,
    plan_type: Option<&'a str>,
    error_msg: Option<&'a str>,
}

pub fn publish_quota_snapshot(app: &AppHandle, config: &QuotaConfig) -> Result<(), String> {
    let app_config =
        crate::config_store::read_config::<crate::models::AppConfig>(app, "app_config.json");
    let language = app_config.language.as_deref().unwrap_or("zh-CN");
    let snapshot = QuotaSnapshot {
        language,
        show_plan_type: config.show_plan_type.unwrap_or(true),
        items: config.items.iter().map(display_item).collect(),
    };

    let home = app.path().home_dir().map_err(|error| error.to_string())?;
    let dir = home.join("Library/Group Containers").join(GROUP_ID);
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let destination = dir.join(SNAPSHOT_FILE);
    let temporary = dir.join(format!("{SNAPSHOT_FILE}.tmp"));
    let bytes = serde_json::to_vec(&snapshot).map_err(|error| error.to_string())?;
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, destination).map_err(|error| error.to_string())
}

fn display_item(item: &QuotaItem) -> QuotaSnapshotItem<'_> {
    QuotaSnapshotItem {
        name: &item.name,
        provider: &item.provider,
        current_value: item.current_value,
        max_quota: item.max_quota,
        unit: item.unit.as_deref(),
        primary_name: item.primary_name.as_deref(),
        primary_reset: item.primary_reset.as_deref(),
        last_update: item.last_update.as_deref(),
        plan_type: item.plan_type.as_deref(),
        error_msg: item.error_msg.as_deref(),
    }
}
