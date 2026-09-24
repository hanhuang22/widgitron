use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Local, TimeZone, Utc};
use serde_json::Value;

use crate::models::{
    QuotaAnalytics, QuotaConfig, QuotaDailyPoint, QuotaItem, QuotaModelBreakdown,
    QuotaSpendSummary, QuotaTokenSummary, QuotaVisualizationConfig,
};

const CURSOR_EVENT_MAX_PAGES: u32 = 50;
const CURSOR_EVENT_PAGE_SIZE: u32 = 200;
const HEATMAP_DAYS: i64 = 84;
const DAILY_BAR_DAYS: i64 = 14;
const CURSOR_LOCAL_TRACKING_LIMIT: i64 = 50_000;

pub fn visualizations_enabled(settings: &QuotaVisualizationConfig) -> bool {
    settings.any_enabled()
}

pub fn item_visualizations(item: &QuotaItem, config: &QuotaConfig) -> QuotaVisualizationConfig {
    item.visualizations
        .clone()
        .or_else(|| config.visualizations.clone())
        .unwrap_or_default()
}

fn or_flag(dst: &mut Option<bool>, src: Option<bool>) {
    if src == Some(true) {
        *dst = Some(true);
    }
}

pub fn merged_visualizations_for_provider(
    config: &QuotaConfig,
    provider: &str,
) -> QuotaVisualizationConfig {
    let mut merged = QuotaVisualizationConfig::default();
    let mut found = false;
    for item in &config.items {
        if item.provider != provider {
            continue;
        }
        found = true;
        let v = item_visualizations(item, config);
        or_flag(&mut merged.spend_summary, v.spend_summary);
        or_flag(&mut merged.calendar_heatmap, v.calendar_heatmap);
        or_flag(&mut merged.daily_bars, v.daily_bars);
        or_flag(&mut merged.model_breakdown, v.model_breakdown);
        or_flag(&mut merged.token_summary, v.token_summary);
        if merged.heatmap_metric.is_none() {
            merged.heatmap_metric = v
                .heatmap_metric
                .clone()
                .or_else(|| v.daily_metric.clone());
        }
        if merged.bars_metric.is_none() {
            merged.bars_metric = v.bars_metric.clone().or_else(|| v.daily_metric.clone());
        }
        if merged.daily_metric.is_none() {
            merged.daily_metric = v.daily_metric.clone();
        }
        if merged.spend_metric.is_none() {
            merged.spend_metric = v.spend_metric.clone();
        }
        if merged.model_metric.is_none() {
            merged.model_metric = v.model_metric.clone();
        }
    }
    if !found {
        return config.visualizations.clone().unwrap_or_default();
    }
    merged
}


fn format_local_from_epoch_ms(ms: i64) -> Option<String> {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
}

fn date_from_epoch_ms(ms: i64) -> Option<String> {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
}

fn billing_cycle_progress(start_ms: i64, end_ms: i64) -> Option<f64> {
    let now = Utc::now().timestamp_millis();
    if end_ms <= start_ms {
        return None;
    }
    Some(((now - start_ms) as f64 / (end_ms - start_ms) as f64 * 100.0).clamp(0.0, 100.0))
}

fn push_daily(map: &mut BTreeMap<String, f64>, date: &str, delta: f64) {
    *map.entry(date.to_string()).or_insert(0.0) += delta;
}

fn daily_map_to_vec(map: BTreeMap<String, f64>, days: i64) -> Vec<QuotaDailyPoint> {
    let today = Local::now().date_naive();
    let mut points = Vec::new();
    for offset in (0..days).rev() {
        let date = today - chrono::Duration::days(offset);
        let key = date.format("%Y-%m-%d").to_string();
        points.push(QuotaDailyPoint {
            date: key.clone(),
            value: *map.get(&key).unwrap_or(&0.0),
            label: None,
        });
    }
    points
}

fn fill_heatmap_days(map: BTreeMap<String, f64>) -> Vec<QuotaDailyPoint> {
    daily_map_to_vec(map, HEATMAP_DAYS)
}

fn fill_daily_bars(map: BTreeMap<String, f64>) -> Vec<QuotaDailyPoint> {
    daily_map_to_vec(map, DAILY_BAR_DAYS)
}

fn short_model_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }
    if trimmed.len() <= 28 {
        return trimmed.to_string();
    }
    format!("{}…", &trimmed[..25])
}

fn parse_u64_field(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .or_else(|| value.as_i64().map(|v| v.max(0) as u64))
}

fn heatmap_start_ms() -> i64 {
    let today = Local::now().date_naive();
    let start = today - chrono::Duration::days(HEATMAP_DAYS - 1);
    start
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).single())
        .map(|dt| dt.timestamp_millis())
        .unwrap_or_else(|| Local::now().timestamp_millis())
}

fn parse_timestamp_ms(value: &Value) -> Option<i64> {
    if let Some(ms) = value.as_i64() {
        return Some(ms);
    }
    if let Some(text) = value.as_str() {
        if let Ok(ms) = text.parse::<i64>() {
            return Some(ms);
        }
        if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
            return Some(dt.with_timezone(&Utc).timestamp_millis());
        }
    }
    None
}

fn cursor_event_tokens(usage: &CursorTokenUsage) -> f64 {
    usage.input_tokens.unwrap_or(0) as f64
        + usage.output_tokens.unwrap_or(0) as f64
        + usage.cache_read_tokens.unwrap_or(0) as f64
}

fn aggregation_token_total(entry: &CursorAggregation) -> Option<u64> {
    let input = entry
        .input_tokens
        .as_deref()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let output = entry
        .output_tokens
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let cache = entry
        .cache_read_tokens
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let total = input + output + cache;
    if total == 0 {
        None
    } else {
        Some(total)
    }
}

fn parse_f64_field(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

// ─── Cursor ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct CursorAggregatedResponse {
    #[serde(default)]
    aggregations: Vec<CursorAggregation>,
    #[serde(rename = "totalInputTokens", default)]
    total_input_tokens: Option<String>,
    #[serde(rename = "totalOutputTokens", default)]
    total_output_tokens: Option<String>,
    #[serde(rename = "totalCacheReadTokens", default)]
    total_cache_read_tokens: Option<String>,
    #[serde(rename = "totalCacheWriteTokens", default)]
    total_cache_write_tokens: Option<String>,
    #[serde(rename = "totalCostCents", default)]
    total_cost_cents: Option<f64>,
}

#[derive(serde::Deserialize)]
struct CursorAggregation {
    #[serde(rename = "modelIntent", default)]
    model_intent: Option<String>,
    #[serde(rename = "inputTokens", default)]
    input_tokens: Option<String>,
    #[serde(rename = "outputTokens", default)]
    output_tokens: Option<String>,
    #[serde(rename = "cacheReadTokens", default)]
    cache_read_tokens: Option<String>,
    #[serde(rename = "totalCents", default)]
    total_cents: Option<f64>,
}

#[derive(serde::Deserialize)]
struct CursorFilteredEventsResponse {
    #[serde(rename = "usageEventsDisplay", default)]
    usage_events_display: Vec<CursorUsageEvent>,
}

#[derive(serde::Deserialize)]
struct CursorUsageEvent {
    timestamp: Option<String>,
    #[serde(rename = "chargedCents", default)]
    charged_cents: Option<f64>,
    #[serde(rename = "tokenUsage", default)]
    token_usage: Option<CursorTokenUsage>,
}

#[derive(serde::Deserialize)]
struct CursorTokenUsage {
    #[serde(rename = "inputTokens", default)]
    input_tokens: Option<u64>,
    #[serde(rename = "outputTokens", default)]
    output_tokens: Option<u64>,
    #[serde(rename = "cacheReadTokens", default)]
    cache_read_tokens: Option<u64>,
}

pub fn cursor_spend_from_plan_usage(
    total_spend: Option<f64>,
    included_spend: Option<f64>,
    bonus_spend: Option<f64>,
    limit: Option<f64>,
    billing_start: Option<String>,
    billing_end: Option<String>,
) -> QuotaSpendSummary {
    let spent = total_spend.unwrap_or(0.0);
    let limit_cents = limit;
    let remaining = limit_cents.map(|l| (l - included_spend.unwrap_or(spent)).max(0.0));

    let start_ms = billing_start
        .as_deref()
        .and_then(|s| s.parse::<i64>().ok());
    let end_ms = billing_end.as_deref().and_then(|s| s.parse::<i64>().ok());

    QuotaSpendSummary {
        spent_cents: spent,
        limit_cents,
        included_cents: included_spend,
        bonus_cents: bonus_spend,
        remaining_cents: remaining,
        billing_cycle_start: start_ms.and_then(format_local_from_epoch_ms),
        billing_cycle_end: end_ms.and_then(format_local_from_epoch_ms),
        billing_cycle_progress_pct: match (start_ms, end_ms) {
            (Some(s), Some(e)) => billing_cycle_progress(s, e),
            _ => None,
        },
        currency: Some("USD".to_string()),
    }
}

async fn cursor_post_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    token: &str,
    path: &str,
    body: &str,
) -> Result<T, String> {
    let res = client
        .post(format!("https://api2.cursor.sh{path}"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("connect-protocol-version", "1")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Cursor analytics network error: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("Cursor analytics HTTP {}", res.status()));
    }

    let text = res
        .text()
        .await
        .map_err(|e| format!("Cursor analytics read error: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("Cursor analytics parse error: {e}"))
}

async fn fetch_cursor_daily_maps(
    client: &reqwest::Client,
    token: &str,
) -> (BTreeMap<String, f64>, BTreeMap<String, f64>) {
    let start_ms = heatmap_start_ms();
    let end_ms = Local::now().timestamp_millis();
    let mut queries: BTreeMap<String, f64> = BTreeMap::new();
    let mut tokens: BTreeMap<String, f64> = BTreeMap::new();

    for page in 1..=CURSOR_EVENT_MAX_PAGES {
        let body = format!(
            r#"{{"page":{page},"pageSize":{CURSOR_EVENT_PAGE_SIZE},"startDate":"{start_ms}","endDate":"{end_ms}"}}"#
        );
        let Ok(resp) = cursor_post_json::<CursorFilteredEventsResponse>(
            client,
            token,
            "/aiserver.v1.DashboardService/GetFilteredUsageEvents",
            &body,
        )
        .await
        else {
            break;
        };

        if resp.usage_events_display.is_empty() {
            break;
        }

        let event_count = resp.usage_events_display.len();
        for event in &resp.usage_events_display {
            let Some(ts) = event
                .timestamp
                .as_deref()
                .and_then(|s| s.parse::<i64>().ok())
            else {
                continue;
            };
            let Some(date) = date_from_epoch_ms(ts) else {
                continue;
            };
            push_daily(&mut queries, &date, 1.0);
            if let Some(usage) = event.token_usage.as_ref() {
                push_daily(&mut tokens, &date, cursor_event_tokens(usage));
            }
        }

        if event_count < CURSOR_EVENT_PAGE_SIZE as usize {
            break;
        }
    }

    merge_cursor_local_tracking(&mut queries);

    (queries, tokens)
}

fn merge_cursor_local_tracking(queries: &mut BTreeMap<String, f64>) {
    let local = read_cursor_local_daily_activity();
    for (date, count) in local {
        push_daily(queries, &date, count);
    }
}

async fn fetch_cursor_daily_activity(
    client: &reqwest::Client,
    token: &str,
) -> BTreeMap<String, f64> {
    fetch_cursor_daily_maps(client, token).await.0
}

fn read_cursor_local_daily_activity() -> BTreeMap<String, f64> {
    let mut daily: BTreeMap<String, f64> = BTreeMap::new();
    let candidates = cursor_tracking_db_paths();
    for path in candidates {
        if !path.exists() {
            continue;
        }
        let Ok(conn) = rusqlite::Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) else {
            continue;
        };
        let mut stmt = match conn.prepare(&format!(
            "SELECT timestamp FROM ai_code_hashes WHERE timestamp IS NOT NULL ORDER BY timestamp DESC LIMIT {CURSOR_LOCAL_TRACKING_LIMIT}"
        )) {
            Ok(stmt) => stmt,
            Err(_) => continue,
        };
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0));
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                if let Some(date) = date_from_epoch_ms(row) {
                    push_daily(&mut daily, &date, 1.0);
                }
            }
        }
        if !daily.is_empty() {
            break;
        }
    }
    daily
}

fn cursor_tracking_db_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        paths.push(
            PathBuf::from(&home)
                .join(".cursor")
                .join("ai-tracking")
                .join("ai-code-tracking.db"),
        );
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        paths.push(
            PathBuf::from(appdata)
                .join("Cursor")
                .join("ai-tracking")
                .join("ai-code-tracking.db"),
        );
    }
    paths
}

async fn fetch_cursor_model_breakdown(
    client: &reqwest::Client,
    token: &str,
) -> (Vec<QuotaModelBreakdown>, Option<QuotaTokenSummary>, Option<f64>) {
    let Ok(resp) = cursor_post_json::<CursorAggregatedResponse>(
        client,
        token,
        "/aiserver.v1.DashboardService/GetAggregatedUsageEvents",
        "{}",
    )
    .await
    else {
        return (Vec::new(), None, None);
    };

    let mut models: Vec<QuotaModelBreakdown> = resp
        .aggregations
        .into_iter()
        .filter_map(|entry| {
            let model = entry
                .model_intent
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let value = entry.total_cents.unwrap_or(0.0);
            if value <= 0.0 && entry.input_tokens.is_none() {
                return None;
            }
            Some(QuotaModelBreakdown {
                model: short_model_name(&model),
                value,
                input_tokens: entry
                    .input_tokens
                    .as_deref()
                    .and_then(|s| s.parse().ok()),
                output_tokens: entry
                    .output_tokens
                    .as_deref()
                    .and_then(|s| s.parse().ok()),
                cache_read_tokens: entry
                    .cache_read_tokens
                    .as_deref()
                    .and_then(|s| s.parse().ok()),
                token_total: aggregation_token_total(&entry),
            })
        })
        .collect();

    models.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    models.truncate(8);

    let tokens = QuotaTokenSummary {
        input_tokens: resp
            .total_input_tokens
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        output_tokens: resp
            .total_output_tokens
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        cache_read_tokens: resp
            .total_cache_read_tokens
            .as_deref()
            .and_then(|s| s.parse().ok()),
        cache_write_tokens: resp
            .total_cache_write_tokens
            .as_deref()
            .and_then(|s| s.parse().ok()),
        total_tokens: None,
        session_count: None,
        request_count: None,
    };

    (
        models,
        Some(tokens),
        resp.total_cost_cents,
    )
}

pub async fn build_cursor_analytics(
    token: &str,
    settings: &QuotaVisualizationConfig,
) -> QuotaAnalytics {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap_or_default();

    let mut analytics = QuotaAnalytics::default();

    let need_daily = settings.calendar_heatmap.unwrap_or(false)
        || settings.daily_bars.unwrap_or(false);
    let need_models = settings.model_breakdown.unwrap_or(false);

    if need_daily {
        let (queries, tokens) = fetch_cursor_daily_maps(&client, token).await;
        analytics.daily_queries = fill_heatmap_days(queries);
        analytics.daily_tokens = fill_heatmap_days(tokens);
    }

    if need_models {
        let (models, _tokens, _total_cost) =
            fetch_cursor_model_breakdown(&client, token).await;
        analytics.model_breakdown = models;
    }

    analytics
}

// ─── Claude Code ────────────────────────────────────────────────────────────

fn claude_project_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        roots.push(PathBuf::from(&home).join(".claude").join("projects"));
        roots.push(PathBuf::from(&home).join(".config").join("claude").join("projects"));
    }
    roots
}

fn collect_claude_jsonl_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in claude_project_roots() {
        if !root.is_dir() {
            continue;
        }
        collect_jsonl_recursive(&root, &mut files);
    }
    files
}

fn collect_jsonl_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_recursive(&path, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
}

struct ClaudeJsonlStats {
    daily_requests: BTreeMap<String, f64>,
    daily_tokens: BTreeMap<String, f64>,
    model_tokens: HashMap<String, f64>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    request_count: u64,
    session_ids: HashMap<String, ()>,
    seen_message_ids: HashMap<String, ()>,
}

impl Default for ClaudeJsonlStats {
    fn default() -> Self {
        Self {
            daily_requests: BTreeMap::new(),
            daily_tokens: BTreeMap::new(),
            model_tokens: HashMap::new(),
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            request_count: 0,
            session_ids: HashMap::new(),
            seen_message_ids: HashMap::new(),
        }
    }
}

fn claude_config_dir() -> PathBuf {
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        return PathBuf::from(home).join(".claude");
    }
    PathBuf::from(shellexpand::tilde("~/.claude").to_string())
}

fn claude_history_path() -> PathBuf {
    claude_config_dir().join("history.jsonl")
}

fn ingest_claude_history_line(stats: &mut ClaudeJsonlStats, line: &str) {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return;
    };
    let Some(ts) = value.get("timestamp").and_then(parse_timestamp_ms) else {
        return;
    };
    let Some(date) = date_from_epoch_ms(ts) else {
        return;
    };
    push_daily(&mut stats.daily_requests, &date, 1.0);
}

fn read_claude_history_stats(stats: &mut ClaudeJsonlStats) {
    let path = claude_history_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        ingest_claude_history_line(stats, line);
    }
}

fn claude_message_id(value: &Value) -> Option<String> {
    value
        .pointer("/message/id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| value.get("uuid").and_then(|v| v.as_str()).map(|s| s.to_string()))
}

fn claude_is_assistant_line(value: &Value) -> bool {
    if value.get("type").and_then(|v| v.as_str()) == Some("assistant") {
        return true;
    }
    value
        .pointer("/message/role")
        .and_then(|v| v.as_str())
        == Some("assistant")
}

fn ingest_claude_jsonl_line(stats: &mut ClaudeJsonlStats, line: &str) {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return;
    };

    if !claude_is_assistant_line(&value) {
        return;
    }

    if let Some(message_id) = claude_message_id(&value) {
        if stats.seen_message_ids.contains_key(&message_id) {
            return;
        }
        stats.seen_message_ids.insert(message_id, ());
    }

    let timestamp = value
        .get("timestamp")
        .and_then(parse_timestamp_ms)
        .and_then(|ms| Local.timestamp_millis_opt(ms).single());

    let Some(ts) = timestamp else {
        return;
    };
    let date = ts.format("%Y-%m-%d").to_string();

    let usage = value
        .pointer("/message/usage")
        .or_else(|| value.get("usage"));

    let Some(usage) = usage else {
        push_daily(&mut stats.daily_requests, &date, 1.0);
        stats.request_count += 1;
        return;
    };

    let input = parse_u64_field(&usage["input_tokens"]).unwrap_or(0);
    let output = parse_u64_field(&usage["output_tokens"]).unwrap_or(0);
    let cache_read = parse_u64_field(&usage["cache_read_input_tokens"]).unwrap_or(0);
    let cache_write = parse_u64_field(&usage["cache_creation_input_tokens"]).unwrap_or(0);
    let token_total = (input + output + cache_read + cache_write) as f64;

    stats.input_tokens += input;
    stats.output_tokens += output;
    stats.cache_read_tokens += cache_read;
    stats.cache_write_tokens += cache_write;
    stats.request_count += 1;
    push_daily(&mut stats.daily_requests, &date, 1.0);
    push_daily(&mut stats.daily_tokens, &date, token_total);

    if let Some(session_id) = value.get("sessionId").and_then(|v| v.as_str()) {
        stats.session_ids.insert(session_id.to_string(), ());
    }

    let model = value
        .pointer("/message/model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let model_key = short_model_name(model);
    *stats.model_tokens.entry(model_key).or_insert(0.0) += token_total;
}

fn merge_claude_daily_requests(
    history: &BTreeMap<String, f64>,
    jsonl: &BTreeMap<String, f64>,
) -> BTreeMap<String, f64> {
    let mut merged = history.clone();
    for (date, count) in jsonl {
        merged
            .entry(date.clone())
            .and_modify(|v| *v = v.max(*count))
            .or_insert(*count);
    }
    merged
}

fn read_claude_jsonl_stats() -> ClaudeJsonlStats {
    let mut history_stats = ClaudeJsonlStats::default();
    read_claude_history_stats(&mut history_stats);
    let history_daily = history_stats.daily_requests;

    let mut stats = ClaudeJsonlStats::default();
    for file in collect_claude_jsonl_files() {
        let Ok(content) = std::fs::read_to_string(&file) else {
            continue;
        };
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            ingest_claude_jsonl_line(&mut stats, line);
        }
    }
    stats.daily_requests = merge_claude_daily_requests(&history_daily, &stats.daily_requests);
    stats
}

pub fn claude_spend_from_usage_json(parsed: &Value) -> Option<QuotaSpendSummary> {
    let spend = parsed.get("spend")?;
    let used_minor = spend
        .pointer("/used/amount_minor")
        .and_then(parse_f64_field)
        .unwrap_or(0.0);
    let exponent = spend
        .pointer("/used/exponent")
        .and_then(parse_f64_field)
        .unwrap_or(2.0);
    let divisor = 10_f64.powf(exponent);
    let spent = if divisor > 0.0 {
        used_minor / divisor
    } else {
        used_minor
    };

    let limit = spend.get("cap").and_then(parse_f64_field);

    Some(QuotaSpendSummary {
        spent_cents: spent * 100.0,
        limit_cents: limit.map(|v| v * 100.0),
        included_cents: None,
        bonus_cents: None,
        remaining_cents: None,
        billing_cycle_start: None,
        billing_cycle_end: None,
        billing_cycle_progress_pct: spend.get("percent").and_then(parse_f64_field),
        currency: spend
            .pointer("/used/currency")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

pub fn build_claude_analytics(settings: &QuotaVisualizationConfig) -> QuotaAnalytics {
    let need_daily = settings.calendar_heatmap.unwrap_or(false)
        || settings.daily_bars.unwrap_or(false);
    let need_jsonl = need_daily || settings.model_breakdown.unwrap_or(false);

    let jsonl_stats = if need_jsonl {
        Some(read_claude_jsonl_stats())
    } else {
        None
    };

    let mut analytics = QuotaAnalytics::default();

    if let Some(stats) = jsonl_stats {
        if need_daily {
            analytics.daily_queries = fill_heatmap_days(stats.daily_requests);
            analytics.daily_tokens = fill_heatmap_days(stats.daily_tokens);
        }

        if settings.model_breakdown.unwrap_or(false) {
            let mut models: Vec<QuotaModelBreakdown> = stats
                .model_tokens
                .into_iter()
                .map(|(model, token_total)| QuotaModelBreakdown {
                    model,
                    value: token_total,
                    input_tokens: None,
                    output_tokens: None,
                    cache_read_tokens: None,
                    token_total: Some(token_total as u64),
                })
                .collect();
            models.sort_by(|a, b| {
                b.value
                    .partial_cmp(&a.value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            models.truncate(8);
            analytics.model_breakdown = models;
        }
    }

    analytics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_spend_summary_computes_remaining() {
        let spend = cursor_spend_from_plan_usage(
            Some(40224.0),
            Some(40000.0),
            Some(224.0),
            Some(40000.0),
            Some("1787907821000".to_string()),
            Some("1790586221000".to_string()),
        );
        assert_eq!(spend.spent_cents, 40224.0);
        assert_eq!(spend.remaining_cents, Some(0.0));
        assert!(spend.billing_cycle_progress_pct.is_some());
    }

    #[test]
    fn claude_jsonl_line_updates_stats() {
        let line = r#"{"type":"assistant","timestamp":"2026-09-02T01:00:00Z","sessionId":"abc","message":{"model":"claude-sonnet-4","usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":10,"cache_creation_input_tokens":5}}}"#;
        let mut stats = ClaudeJsonlStats::default();
        ingest_claude_jsonl_line(&mut stats, line);
        assert_eq!(stats.request_count, 1);
        assert_eq!(stats.input_tokens, 100);
        assert_eq!(stats.output_tokens, 50);
        assert_eq!(stats.session_ids.len(), 1);
    }

    #[test]
    fn claude_history_line_counts_daily_request() {
        let line = r#"{"timestamp":1756771200000,"text":"hello"}"#;
        let mut stats = ClaudeJsonlStats::default();
        ingest_claude_history_line(&mut stats, line);
        assert_eq!(stats.daily_requests.len(), 1);
    }

    #[test]
    fn claude_jsonl_dedupes_message_id() {
        let line = r#"{"type":"assistant","timestamp":"2026-09-02T01:00:00Z","message":{"id":"msg-1","model":"claude-sonnet-4","usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let mut stats = ClaudeJsonlStats::default();
        ingest_claude_jsonl_line(&mut stats, line);
        ingest_claude_jsonl_line(&mut stats, line);
        assert_eq!(stats.request_count, 1);
    }
}
