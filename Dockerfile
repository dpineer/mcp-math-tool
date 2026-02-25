# 多阶段构建 - 构建阶段
FROM rust:1.84-slim AS builder

# 设置工作目录
WORKDIR /usr/src/app

# 复制依赖文件并预构建依赖（利用Docker缓存）
COPY Cargo.toml Cargo.lock ./

# 创建一个临时的src目录并复制main.rs以构建依赖
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建最终的可执行文件
RUN cargo build --release

# 运行阶段 - 使用更小的基础镜像
FROM debian:bookworm-slim

# 安装必要的运行时依赖
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    curl \
    netcat-openbsd \
    procps \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /usr/local/bin

# 从构建阶段复制可执行文件
COPY --from=builder /usr/src/app/target/release/mcp-math-tool ./

# 暴露端口（HTTP服务器默认端口）
EXPOSE 3000

# 启动命令 - 明确指定运行HTTP服务器模式
CMD ["./mcp-math-tool", "http", "--addr", "0.0.0.0:3000"]
