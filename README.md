# IPTV Speed Tester（Rust 版）

自动聚合、测速、过滤 IPTV 源，输出可直接导入播放器的 M3U8 / TXT 播放列表。

---

## 功能

- 从 API 接口获取公共 IPTV 主机，并发测速，保留高质量源
- 支持导入自定义订阅（M3U / TXT 格式均可）
- 自动清洗频道名（CCTV 别名归一、卫视排序等）
- 按速度分级（高速 ≥5 MB/s / 普通 ≥1 MB/s / 低速）分组输出
- 定时自动更新，HTTP 接口实时提供播放列表
- 全程异步并发（Tokio），比 Go 版内存占用更低

---

## HTTP 接口

| 路径 | 说明 |
|---|---|
| `GET /` 或 `/iptv` | M3U8 播放列表 |
| `GET /txt` | TXT 格式播放列表（逗号分隔） |
| `GET /status` | JSON 状态（运行状态 / 上次更新时间） |
| `GET /retest` | 立即触发重新测速（异步，不阻塞） |

---

## 快速开始

### 方式一：本地 Rust 编译

**前置要求**：Rust 1.70+、OpenSSL 开发库

```bash
# Ubuntu / Debian
sudo apt install pkg-config libssl-dev

# macOS
brew install openssl

# 编译
./build.sh

# 运行（默认端口 3030）
./build.sh run

# 带自定义订阅运行
URL1=http://your-sub-url ./build.sh run
```

### 方式二：Docker

```bash
# 构建镜像并启动容器
./build.sh docker-run

# 或者指定订阅源
URL1=http://your-sub ./build.sh docker-run
```

### 方式三：docker-compose（推荐生产环境）

```bash
# 编辑 docker-compose.yml，在 environment 里填写订阅地址
# 然后启动：
./build.sh compose

# 或直接：
docker compose up -d --build
```

---

## 配置参数

所有参数既可以通过命令行标志，也可以通过环境变量设置。

| 参数 | 环境变量 | 默认值 | 说明 |
|---|---|---|---|
| `--port` | `PORT` | `3030` | HTTP 监听端口 |
| `--workers` | `WORKERS` | `20` | 并发测速数 |
| `--top` | `TOP` | `5` | 每类型保留最优源数 |
| `--interval` | `INTERVAL` | `6h` | 自动更新间隔（6h / 30m / 3600s）|
| `--url1`..`--url20` | `URL1`..`URL20` | — | 自定义订阅源 |

---

## 自定义文件

将以下文件放在工作目录（或 Docker 的 `/app/data` 挂载目录）中：

| 文件 | 说明 |
|---|---|
| `channel_list.txt` | 标准频道名列表，每行一个，用于名称归一化 |
| `hsmd_address_list.txt` | 和事猫TV频道地址列表 |

---

## 目录结构

```
iptv-speed-tester/
├── src/
│   ├── main.rs        # 入口、CLI、HTTP 路由
│   ├── config.rs      # 全局常量
│   ├── types.rs       # 数据结构
│   ├── channel.rs     # 频道名清洗 / 分组 / 排序
│   ├── speedtest.rs   # 异步测速核心
│   ├── subscribe.rs   # 订阅文件下载 & 解析
│   ├── task.rs        # 主调度任务
│   ├── server.rs      # HTTP 处理器
│   └── output.rs      # M3U8 / TXT 生成 & 缓存
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── build.sh
└── README.md
```

---

## 从 Go 版迁移的主要变化

| Go | Rust |
|---|---|
| `goroutine` + `sync.WaitGroup` | `tokio::spawn` + `JoinHandle` |
| `sync.RWMutex` | `tokio::sync::RwLock` |
| channel 信号量 | `tokio::sync::Semaphore` |
| `net/http` | `axum` |
| `encoding/json` | `serde_json` |
| `flag` + `os.Getenv` | `clap`（内置 env 支持） |
| 全局 `var` + mutex | `once_cell::Lazy` |
