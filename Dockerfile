# ── Stage 1: Builder ─────────────────────────────────────────────
FROM rust:1.82-slim-bookworm AS builder

# 安装编译依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 先只复制依赖声明，利用 Docker 层缓存加速重复构建
COPY Cargo.toml Cargo.lock* ./

# 构建一个只含 main 的空项目，缓存所有依赖编译结果
RUN mkdir src && echo 'fn main(){}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

# 再复制真正的源码并编译
COPY src ./src
# 触发重新编译（修改 src/main.rs 时间戳）
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ─────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Asia/Shanghai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

WORKDIR /app

# 从 builder 拷贝编译好的二进制
COPY --from=builder /app/target/release/iptv-speed-tester /usr/local/bin/iptv-speed-tester

# 创建数据目录（存放缓存文件和频道列表）
RUN mkdir -p /app/data
WORKDIR /app/data

# 可选：预置频道列表文件（挂载覆盖同名文件即可自定义）
# COPY channel_list.txt /app/data/channel_list.txt
# COPY hsmd_address_list.txt /app/data/hsmd_address_list.txt

EXPOSE 3030

ENTRYPOINT ["/usr/local/bin/iptv-speed-tester"]
