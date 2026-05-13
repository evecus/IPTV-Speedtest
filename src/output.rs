use crate::channel::{base_group, build_m3u8_entry, channel_sort_key, speed_tier, tier_order};
use crate::config::{CACHE_M3U8, CACHE_TXT, EPG_URL};
use crate::types::Entry;
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::fs;

/// 所有速度分级分组（固定顺序）
static TIER_GROUPS: &[(&str, &str, &str)] = &[
    ("央视", "高速", "央视（高速）"),
    ("央视", "普通", "央视（普通）"),
    ("央视", "低速", "央视（低速）"),
    ("卫视", "高速", "卫视（高速）"),
    ("卫视", "普通", "卫视（普通）"),
    ("卫视", "低速", "卫视（低速）"),
    ("其他", "高速", "其他（高速）"),
    ("其他", "普通", "其他（普通）"),
    ("其他", "低速", "其他（低速）"),
];

/// 聚合所有条目、去重、排序，写入文件，返回 (m3u8, txt)
pub fn build_and_write(all_entries: Vec<Entry>, update_time: chrono::DateTime<chrono::Local>) -> (String, String) {
    // ── 按频道名分组 ─────────────────────────────────────────────
    let mut by_name: HashMap<String, Vec<Entry>> = HashMap::new();
    for e in all_entries {
        by_name.entry(e.name.clone()).or_default().push(e);
    }

    // ── 排序频道名 ───────────────────────────────────────────────
    let mut all_names: Vec<String> = by_name.keys().cloned().collect();
    all_names.sort_by(|a, b| {
        let (a0, a1, a2) = channel_sort_key(a);
        let (b0, b1, b2) = channel_sort_key(b);
        a0.cmp(&b0)
            .then(a1.partial_cmp(&b1).unwrap_or(std::cmp::Ordering::Equal))
            .then(a2.cmp(&b2))
    });

    // ── 每个频道去重并按分级+索引排序 ───────────────────────────
    for entries in by_name.values_mut() {
        let mut seen = std::collections::HashSet::new();
        entries.retain(|e| seen.insert(e.url.clone()));
        entries.sort_by(|a, b| {
            let ta = tier_order(speed_tier(a.speed));
            let tb = tier_order(speed_tier(b.speed));
            ta.cmp(&tb).then(a.index.cmp(&b.index))
        });
    }

    let ts = update_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let dummy_name = format!("更新时间: {}", ts);
    const DUMMY_URL: &str = "http://127.0.0.1/";

    // ── M3U8 ─────────────────────────────────────────────────────
    let mut m3u8_lines: Vec<String> = vec![
        format!("#EXTM3U x-tvg-url=\"{}\"", EPG_URL),
        format!("#EXT-X-UPDATED: {}", ts),
    ];
    for name in &all_names {
        if let Some(entries) = by_name.get(name) {
            for e in entries {
                m3u8_lines.push(e.content.clone());
            }
        }
    }
    m3u8_lines.push(format!(
        "#EXTINF:-1 group-title=\"更新时间\",{}\n{}",
        dummy_name, DUMMY_URL
    ));
    let m3u8 = m3u8_lines.join("\n");
    let _ = fs::write(CACHE_M3U8, &m3u8);

    // ── TXT ──────────────────────────────────────────────────────
    let mut group_lines: HashMap<String, Vec<String>> = HashMap::new();
    for name in &all_names {
        if let Some(entries) = by_name.get(name) {
            for e in entries {
                let label = format!("{}（{}）", base_group(&e.name), speed_tier(e.speed));
                group_lines
                    .entry(label)
                    .or_default()
                    .push(format!("{},{}", e.name, e.url));
            }
        }
    }
    let mut txt_parts: Vec<String> = vec![];
    for (_, _, label) in TIER_GROUPS {
        let lines = group_lines.get(*label).cloned().unwrap_or_default();
        if lines.is_empty() {
            continue;
        }
        txt_parts.push(format!("{},#genre#", label));
        txt_parts.extend(lines);
        txt_parts.push(String::new());
    }
    txt_parts.push("更新时间,#genre#".to_string());
    txt_parts.push(format!("{},{}", dummy_name, DUMMY_URL));
    let txt = txt_parts.join("\n");
    let _ = fs::write(CACHE_TXT, &txt);

    println!(
        "[output] m3u8 {} bytes  txt {} bytes  channels {}",
        m3u8.len(),
        txt.len(),
        all_names.len()
    );
    (m3u8, txt)
}

/// 读取缓存文件
pub fn read_cache() -> (String, String) {
    let m3u8 = fs::read_to_string(CACHE_M3U8).unwrap_or_default();
    let txt = fs::read_to_string(CACHE_TXT).unwrap_or_default();
    (m3u8, txt)
}
