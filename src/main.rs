mod channel;
mod config;
mod output;
mod server;
mod speedtest;
mod subscribe;
mod task;
mod types;

use crate::config::{DEFAULT_SUB_URL, VERSION};
use crate::output::read_cache;
use axum::{routing::get, Router};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── CLI 参数 ──────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(version = VERSION, about = "IPTV Speed Tester & Aggregator")]
struct Cli {
    /// HTTP 监听端口
    #[arg(long, env = "PORT", default_value_t = 3030)]
    port: u16,

    /// 并发测速工作数
    #[arg(long, env = "WORKERS", default_value_t = 20)]
    workers: usize,

    /// 每种类型保留前 N 个源
    #[arg(long = "top", env = "TOP", default_value_t = 5)]
    top_n: usize,

    /// 更新间隔，例如 6h / 30m / 3600s
    #[arg(long, env = "INTERVAL", default_value = "6h")]
    interval: String,

    #[arg(long = "url1",  env = "URL1")]  url1:  Option<String>,
    #[arg(long = "url2",  env = "URL2")]  url2:  Option<String>,
    #[arg(long = "url3",  env = "URL3")]  url3:  Option<String>,
    #[arg(long = "url4",  env = "URL4")]  url4:  Option<String>,
    #[arg(long = "url5",  env = "URL5")]  url5:  Option<String>,
    #[arg(long = "url6",  env = "URL6")]  url6:  Option<String>,
    #[arg(long = "url7",  env = "URL7")]  url7:  Option<String>,
    #[arg(long = "url8",  env = "URL8")]  url8:  Option<String>,
    #[arg(long = "url9",  env = "URL9")]  url9:  Option<String>,
    #[arg(long = "url10", env = "URL10")] url10: Option<String>,
    #[arg(long = "url11", env = "URL11")] url11: Option<String>,
    #[arg(long = "url12", env = "URL12")] url12: Option<String>,
    #[arg(long = "url13", env = "URL13")] url13: Option<String>,
    #[arg(long = "url14", env = "URL14")] url14: Option<String>,
    #[arg(long = "url15", env = "URL15")] url15: Option<String>,
    #[arg(long = "url16", env = "URL16")] url16: Option<String>,
    #[arg(long = "url17", env = "URL17")] url17: Option<String>,
    #[arg(long = "url18", env = "URL18")] url18: Option<String>,
    #[arg(long = "url19", env = "URL19")] url19: Option<String>,
    #[arg(long = "url20", env = "URL20")] url20: Option<String>,
}

impl Cli {
    fn collect_urls(&self) -> Vec<String> {
        let opts: &[&Option<String>] = &[
            &self.url1, &self.url2, &self.url3, &self.url4, &self.url5,
            &self.url6, &self.url7, &self.url8, &self.url9, &self.url10,
            &self.url11, &self.url12, &self.url13, &self.url14, &self.url15,
            &self.url16, &self.url17, &self.url18, &self.url19, &self.url20,
        ];
        let mut urls: Vec<String> = opts.iter().filter_map(|o| o.as_deref().map(str::to_string)).collect();
        // 内置默认订阅（失败时自动跳过）
        urls.push(DEFAULT_SUB_URL.to_string());
        urls
    }
}

// ── 共享状态 ──────────────────────────────────────────────────────

#[derive(Default)]
pub struct SharedData {
    pub m3u8: String,
    pub txt: String,
    pub last_run: String,
}

pub struct AppState {
    pub data: RwLock<SharedData>,
    pub workers: usize,
    pub top_n: usize,
    pub urls: Vec<String>,
}

// ── 程序入口 ──────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // 设置时区（Linux 环境下生效）
    unsafe { std::env::set_var("TZ", "Asia/Shanghai") };

    let cli = Cli::parse();
    let urls = cli.collect_urls();

    println!(
        "IPTV Aggregator v{}  port={}  workers={}  top={}  interval={}",
        VERSION, cli.port, cli.workers, cli.top_n, cli.interval
    );
    println!("Subscribe URLs ({}):", urls.len());
    for (i, u) in urls.iter().enumerate() {
        println!("  {}. {}", i + 1, u);
    }

    // 先恢复缓存，让 HTTP 服务立刻可用
    let (m3u8, txt) = read_cache();
    let state = Arc::new(AppState {
        data: RwLock::new(SharedData {
            m3u8,
            txt,
            last_run: "Never".to_string(),
        }),
        workers: cli.workers,
        top_n: cli.top_n,
        urls: urls.clone(),
    });

    // 首次立即执行任务
    {
        let st = state.clone();
        let us = urls.clone();
        let (w, t) = (cli.workers, cli.top_n);
        tokio::spawn(async move { task::run_task(st, w, t, us).await });
    }

    // 定时任务循环
    {
        let secs = parse_interval(&cli.interval);
        println!("[cron] scheduled every {}s", secs);
        let st = state.clone();
        let us = urls.clone();
        let (w, t) = (cli.workers, cli.top_n);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(secs));
            ticker.tick().await; // 跳过第一次（已手动执行）
            loop {
                ticker.tick().await;
                let st2 = st.clone();
                let us2 = us.clone();
                tokio::spawn(task::run_task(st2, w, t, us2));
            }
        });
    }

    // HTTP 路由
    let app = Router::new()
        .route("/",        get(server::handle_m3u8))
        .route("/iptv",    get(server::handle_m3u8))
        .route("/txt",     get(server::handle_txt))
        .route("/status",  get(server::handle_status))
        .route("/retest",  get(server::handle_force_retest))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cli.port);
    println!("[main] listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 解析时间字符串 → 秒数（"6h" → 21600，"30m" → 1800，"3600" → 3600）
fn parse_interval(s: &str) -> u64 {
    if let Some(n) = s.strip_suffix('h') { return n.parse::<u64>().unwrap_or(6) * 3600; }
    if let Some(n) = s.strip_suffix('m') { return n.parse::<u64>().unwrap_or(360) * 60;  }
    if let Some(n) = s.strip_suffix('s') { return n.parse::<u64>().unwrap_or(21600);      }
    s.parse::<u64>().unwrap_or(6 * 3600)
}
