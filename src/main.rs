use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

// ==========================================
// 1. 数学逻辑核心 (LaTeX 处理)
// ==========================================

/// 将 LaTeX 字符串转换为 meval 库可理解的数学表达式
fn latex_to_math_expr(latex: &str) -> String {
    let mut expr = latex.to_string();

    // 1. 处理分式 \frac{a}{b} -> ((a)/(b))
    let re_frac = Regex::new(r"\\frac\{(.+?)\}\{(.+?)\}").unwrap();
    while re_frac.is_match(&expr) {
        expr = re_frac.replace_all(&expr, "(($1)/($2))").to_string();
    }

    // 2. 处理平方根 \sqrt{a} -> sqrt(a)
    let re_sqrt = Regex::new(r"\\sqrt\{(.+?)\}").unwrap();
    expr = re_sqrt.replace_all(&expr, "sqrt($1)").to_string();

    // 3. 处理乘号 \cdot, \times -> *
    expr = expr.replace(r"\cdot", "*").replace(r"\times", "*");

    // 4. 处理括号 \left( ... \right) -> ( ... )
    expr = expr.replace(r"\left(", "(").replace(r"\right)", ")");

    // 5. 处理特殊常数 \pi -> pi
    expr = expr.replace(r"\pi", "pi");

    // 6. 清理其他 LaTeX 常用符号（视情况添加）
    expr = expr.replace("{", "(").replace("}", ")");
    
    expr
}

/// 执行计算
fn calculate(latex: &str) -> Result<f64> {
    let expr_str = latex_to_math_expr(latex);
    // 使用 meval 进行求值
    let result = meval::eval_str(&expr_str)?;
    Ok(result)
}

// ==========================================
// 2. HTTP API 数据结构
// ==========================================

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

// ==========================================
// 3. HTTP 路由处理器
// ==========================================

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
    
    match calculate(&payload.expression) {
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

// ==========================================
// 4. 主程序 (HTTP Server)
// ==========================================

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("启动 MCP 数学工具 HTTP API 服务...");

    // 创建路由
    let app = Router::new()
        .route("/health", post(health_check))
        .route("/calculate", post(calculate_handler))
        .layer(CorsLayer::permissive()); // 允许所有 CORS 请求

    // 启动服务器
    let addr = "0.0.0.0:3000";
    info!("服务器监听地址: {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
