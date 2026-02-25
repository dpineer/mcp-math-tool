use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber;

mod mcp_server;

/// 命令行参数配置
#[derive(Parser)]
#[command(name = "mcp-math-tool")]
#[command(about = "MCP数学工具 - 支持HTTP API和MCP协议")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand)]
enum Mode {
    /// 运行HTTP服务器模式
    Http {
        /// 监听地址
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        addr: String,
    },
    /// 运行MCP服务器模式
    Mcp {
        /// 标准输入输出模式（默认）
        #[arg(long, default_value = "true")]
        stdio: bool,
    },
}

/// 启动HTTP服务器
async fn run_http_server(addr: String) -> Result<()> {
    use axum::{
        extract::State,
        http::{HeaderMap, StatusCode},
        response::IntoResponse,
        routing::post,
        Json, Router,
    };
    use serde::{Deserialize, Serialize};
    use tower_http::cors::CorsLayer;

    // HTTP API 数据结构
    #[derive(Serialize, Deserialize, Debug)]
    struct CalculateRequest {
        expression: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CalculateResponse {
        result: f64,
        expression: String,
        success: bool,
        error: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct HealthResponse {
        status: String,
        version: String,
        service: String,
    }

    /// 健康检查端点
    async fn health_check() -> impl IntoResponse {
        let response = HealthResponse {
            status: "ok".to_string(),
            version: "1.0.0".to_string(),
            service: "mcp-math-api".to_string(),
        };
        (StatusCode::OK, Json(response))
    }

    /// 计算端点
    async fn calculate_handler(
        headers: HeaderMap,
        Json(payload): Json<CalculateRequest>,
    ) -> impl IntoResponse {
        info!("收到计算请求: {}", payload.expression);
        
        // 记录请求头（用于调试）
        if let Some(user_agent) = headers.get("user-agent") {
            info!("User-Agent: {:?}", user_agent);
        }
        
        match mcp_server::calculate(&payload.expression) {
            Ok(result) => {
                let response = CalculateResponse {
                    result,
                    expression: payload.expression,
                    success: true,
                    error: None,
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                let error_msg = format!("计算错误: {}", e);
                info!("{}", error_msg);
                let response = CalculateResponse {
                    result: 0.0,
                    expression: payload.expression,
                    success: false,
                    error: Some(error_msg),
                };
                (StatusCode::BAD_REQUEST, Json(response))
            }
        }
    }

    info!("启动 MCP 数学工具 HTTP API 服务...");
    info!("服务器监听地址: {}", addr);

    // 创建路由
    let app = Router::new()
        .route("/health", post(health_check))
        .route("/calculate", post(calculate_handler))
        .layer(CorsLayer::permissive()); // 允许所有 CORS 请求

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 启动MCP服务器
async fn run_mcp_server() -> Result<()> {
    info!("启动 MCP 数学工具 MCP 服务器...");
    mcp_server::run_mcp_server().await.map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

/// 数学计算函数（复用原有逻辑）
fn calculate(latex: &str) -> Result<f64> {
    match mcp_server::calculate(latex) {
        Ok(result) => Ok(result),
        Err(e) => Err(anyhow::anyhow!("{}", e))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.mode {
        Some(Mode::Http { addr }) => {
            info!("运行HTTP服务器模式");
            run_http_server(addr).await?;
        }
        Some(Mode::Mcp { stdio: _ }) => {
            info!("运行MCP服务器模式");
            run_mcp_server().await?;
        }
        None => {
            // 默认运行HTTP服务器
            info!("未指定模式，默认运行HTTP服务器模式");
            run_http_server("0.0.0.0:3000".to_string()).await?;
        }
    }

    Ok(())
}