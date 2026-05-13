#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────
# build.sh — IPTV Speed Tester 构建 & 运行辅助脚本
# 用法：
#   ./build.sh              # 本地 Release 构建
#   ./build.sh run          # 本地构建后直接运行
#   ./build.sh docker       # Docker 镜像构建
#   ./build.sh docker-run   # Docker 镜像构建并运行容器
#   ./build.sh compose      # docker-compose 启动
#   ./build.sh clean        # 清理编译产物
# ──────────────────────────────────────────────────────────────────
set -e

APP=iptv-speed-tester
IMAGE=iptv-speed-tester:latest
PORT=${PORT:-3030}

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[build.sh]${NC} $*"; }
ok()    { echo -e "${GREEN}[build.sh]${NC} $*"; }
error() { echo -e "${RED}[build.sh]${NC} $*"; exit 1; }

# ── 检查工具 ──────────────────────────────────────────────────────
need() { command -v "$1" >/dev/null 2>&1 || error "需要 $1，请先安装"; }

# ── 子命令 ───────────────────────────────────────────────────────

cmd_build() {
    need cargo
    info "正在 Release 编译..."
    cargo build --release
    ok "编译完成 → target/release/${APP}"
}

cmd_run() {
    cmd_build
    info "启动本地服务（端口 ${PORT}）..."
    ./target/release/${APP} \
        --port "${PORT}" \
        --workers "${WORKERS:-20}" \
        --top "${TOP:-5}" \
        --interval "${INTERVAL:-6h}" \
        ${URL1:+--url1 "$URL1"} \
        ${URL2:+--url2 "$URL2"} \
        ${URL3:+--url3 "$URL3"} \
        ${URL4:+--url4 "$URL4"} \
        ${URL5:+--url5 "$URL5"}
}

cmd_docker() {
    need docker
    info "构建 Docker 镜像 ${IMAGE}..."
    docker build -t "${IMAGE}" .
    ok "镜像构建完成：${IMAGE}"
}

cmd_docker_run() {
    cmd_docker
    info "启动 Docker 容器（端口 ${PORT}）..."
    docker run -d \
        --name "${APP}" \
        --restart unless-stopped \
        -p "${PORT}:3030" \
        -v "$(pwd)/data:/app/data" \
        -e PORT=3030 \
        -e WORKERS="${WORKERS:-20}" \
        -e TOP="${TOP:-5}" \
        -e INTERVAL="${INTERVAL:-6h}" \
        ${URL1:+-e URL1="$URL1"} \
        ${URL2:+-e URL2="$URL2"} \
        ${URL3:+-e URL3="$URL3"} \
        ${URL4:+-e URL4="$URL4"} \
        ${URL5:+-e URL5="$URL5"} \
        "${IMAGE}"
    ok "容器已启动 → http://localhost:${PORT}"
    echo ""
    echo "  播放列表(m3u8): http://localhost:${PORT}/iptv"
    echo "  播放列表(txt) : http://localhost:${PORT}/txt"
    echo "  状态          : http://localhost:${PORT}/status"
    echo "  立即重测      : http://localhost:${PORT}/retest"
}

cmd_compose() {
    need docker
    info "使用 docker-compose 启动..."
    mkdir -p data
    docker compose up -d --build
    ok "服务已启动 → http://localhost:${PORT}"
}

cmd_clean() {
    need cargo
    info "清理编译产物..."
    cargo clean
    rm -f sub_cache_*.txt
    ok "清理完成"
}

cmd_help() {
    echo "用法: ./build.sh [命令]"
    echo ""
    echo "命令:"
    echo "  (空)         本地 Release 编译"
    echo "  run          本地编译并运行"
    echo "  docker       构建 Docker 镜像"
    echo "  docker-run   构建 Docker 镜像并启动容器"
    echo "  compose      docker-compose 启动"
    echo "  clean        清理编译产物"
    echo ""
    echo "环境变量:"
    echo "  PORT=3030    监听端口"
    echo "  WORKERS=20   并发测速协程数"
    echo "  TOP=5        每类型保留最优源数"
    echo "  INTERVAL=6h  自动更新间隔 (6h/30m/3600s)"
    echo "  URL1..URL5   额外订阅源地址"
}

# ── 入口 ─────────────────────────────────────────────────────────
case "${1:-build}" in
    build)      cmd_build ;;
    run)        cmd_run ;;
    docker)     cmd_docker ;;
    docker-run) cmd_docker_run ;;
    compose)    cmd_compose ;;
    clean)      cmd_clean ;;
    help|-h)    cmd_help ;;
    *) error "未知命令: $1（运行 ./build.sh help 查看帮助）" ;;
esac
