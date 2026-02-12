# 使用较新的 Rust 版本以支持新的 lock file 格式
FROM rust:1.84-slim AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# 安装必要的运行库（OpenSSL 等通常是必须的）
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin
# 注意：确保这里的名称与你 Cargo.toml 中的 name 一致
# 如果你的项目名是 mcp-math-api，请确保路径正确
COPY --from=builder /usr/src/app/target/release/mcp-math-tool . 
EXPOSE 3000
CMD ["./mcp-math-tool"]