use crate::config::{LOGO_BASE_URL, SPEED_HIGH, SPEED_MID};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs;

// ── 速度分级 ──────────────────────────────────────────────────────

pub fn speed_tier(speed: f64) -> &'static str {
    if speed >= SPEED_HIGH {
        "高速"
    } else if speed >= SPEED_MID {
        "普通"
    } else {
        "低速"
    }
}

pub fn tier_order(tier: &str) -> i32 {
    match tier {
        "高速" => 0,
        "普通" => 1,
        _ => 2,
    }
}

// ── 分组 / Logo ───────────────────────────────────────────────────

pub fn base_group(name: &str) -> &'static str {
    let upper = name.to_uppercase();
    if upper.contains("CCTV") {
        "央视"
    } else if name.contains("卫视") {
        "卫视"
    } else {
        "其他"
    }
}

pub fn full_group(name: &str, speed: f64) -> String {
    format!("{}（{}）", base_group(name), speed_tier(speed))
}

pub fn build_logo_url(name: &str) -> String {
    let encoded = url::form_urlencoded::byte_serialize(name.as_bytes())
        .collect::<String>()
        .replace('+', "%20");
    format!("{}{}.png", LOGO_BASE_URL, encoded)
}

pub fn build_m3u8_entry(name: &str, stream_url: &str, speed: f64) -> String {
    let grp = full_group(name, speed);
    format!(
        "#EXTINF:-1 tvg-name=\"{}\" tvg-logo=\"{}\" group-title=\"{}\",{}\n{}",
        name,
        build_logo_url(name),
        grp,
        name,
        stream_url
    )
}

// ── 频道名清洗 ────────────────────────────────────────────────────

static RE_CCTV_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"CCTV(\d+)台").unwrap());

static CCTV_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    let pairs: &[(&str, &str)] = &[
        ("CCTV1综合", "CCTV1"),
        ("CCTV2财经", "CCTV2"),
        ("CCTV3综艺", "CCTV3"),
        ("CCTV4国际", "CCTV4"),
        ("CCTV4中文国际", "CCTV4"),
        ("CCTV4欧洲", "CCTV4"),
        ("CCTV5体育", "CCTV5"),
        ("CCTV6电影", "CCTV6"),
        ("CCTV7军事", "CCTV7"),
        ("CCTV7军农", "CCTV7"),
        ("CCTV7农业", "CCTV7"),
        ("CCTV7国防军事", "CCTV7"),
        ("CCTV8电视剧", "CCTV8"),
        ("CCTV9记录", "CCTV9"),
        ("CCTV9纪录", "CCTV9"),
        ("CCTV10科教", "CCTV10"),
        ("CCTV11戏曲", "CCTV11"),
        ("CCTV12社会与法", "CCTV12"),
        ("CCTV13新闻", "CCTV13"),
        ("CCTV新闻", "CCTV13"),
        ("CCTV14少儿", "CCTV14"),
        ("CCTV15音乐", "CCTV15"),
        ("CCTV16奥林匹克", "CCTV16"),
        ("CCTV17农业农村", "CCTV17"),
        ("CCTV17农业", "CCTV17"),
        ("CCTV5+体育赛视", "CCTV5+"),
        ("CCTV5+体育赛事", "CCTV5+"),
        ("CCTV5+体育", "CCTV5+"),
        ("CCTV01", "CCTV1"),
        ("CCTV02", "CCTV2"),
        ("CCTV03", "CCTV3"),
        ("CCTV04", "CCTV4"),
        ("CCTV05", "CCTV5"),
        ("CCTV06", "CCTV6"),
        ("CCTV07", "CCTV7"),
        ("CCTV08", "CCTV8"),
        ("CCTV09", "CCTV9"),
    ];
    for (k, v) in pairs {
        m.insert(*k, *v);
    }
    m
});

pub fn clean_channel_name(name: &str) -> String {
    let mut s = name.to_string();
    s = s.replace("cctv", "CCTV");
    s = s.replace("中央", "CCTV");
    s = s.replace("央视", "CCTV");
    for rep in &["高清", "超高", "HD", "标清", "频道", "-", " ", "(", ")"] {
        s = s.replace(rep, "");
    }
    s = s.replace("PLUS", "+");
    s = s.replace('＋', "+");
    s = RE_CCTV_NUM
        .replace_all(&s, |caps: &regex::Captures| {
            format!("CCTV{}", &caps[1])
        })
        .into_owned();
    if let Some(&mapped) = CCTV_ALIASES.get(s.as_str()) {
        return mapped.to_string();
    }
    s
}

// ── 标准名映射 ────────────────────────────────────────────────────

pub fn get_standard_channel_map() -> HashMap<String, String> {
    let mut m = HashMap::new();
    let Ok(data) = fs::read_to_string(crate::config::CHANNEL_LIST_FILE) else {
        return m;
    };
    for line in data.lines() {
        let std = line.trim();
        if std.is_empty() {
            continue;
        }
        m.insert(normal_key(std), std.to_string());
    }
    m
}

pub fn normal_key(s: &str) -> String {
    s.to_uppercase().replace('-', "").replace(' ', "")
}

pub fn map_to_standard_name<'a>(name: &'a str, m: &'a HashMap<String, String>) -> &'a str {
    m.get(&normal_key(name)).map(|s| s.as_str()).unwrap_or(name)
}

// ── 卫视排序 ──────────────────────────────────────────────────────

static WEIXI_ORDER: &[&str] = &[
    "湖南卫视", "东方卫视", "浙江卫视", "江苏卫视", "北京卫视", "山东卫视", "河南卫视",
    "广东卫视", "安徽卫视", "深圳卫视", "天津卫视", "江西卫视", "四川卫视", "湖北卫视",
    "重庆卫视", "黑龙江卫视", "辽宁卫视", "河北卫视", "吉林卫视", "山西卫视", "广西卫视",
    "云南卫视", "福建东南卫视", "贵州卫视", "陕西卫视", "甘肃卫视", "内蒙古卫视", "新疆卫视",
    "宁夏卫视", "青海卫视", "西藏卫视", "海南卫视", "兵团卫视",
];

pub fn weixi_sort_index(name: &str) -> Option<usize> {
    WEIXI_ORDER
        .iter()
        .position(|&kw| name.contains(kw))
}

// 返回 (category, sub_order, name) 用于排序
pub fn channel_sort_key(name: &str) -> (i32, f64, String) {
    let upper = name.to_uppercase();
    if upper.contains("CCTV") {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"CCTV(\d+)").unwrap());
        if let Some(caps) = RE.captures(&upper) {
            let num: f64 = caps[1].parse().unwrap_or(999.0);
            return (0, num, String::new());
        }
        if upper.contains("5+") {
            return (0, 5.5, String::new());
        }
        return (0, 999.0, String::new());
    }
    if name.contains("卫视") {
        if let Some(idx) = weixi_sort_index(name) {
            return (1, idx as f64, name.to_string());
        }
        return (1, WEIXI_ORDER.len() as f64, name.to_string());
    }
    (2, 0.0, name.to_string())
}
