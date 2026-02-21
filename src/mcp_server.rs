use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{info, error};

/// MCP请求结构
#[derive(Debug, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

/// MCP响应结构
#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<McpError>,
    id: Option<Value>,
}

/// MCP错误结构
#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// 将 LaTeX 字符串转换为可计算的数学表达式
fn latex_to_math_expr(latex: &str) -> String {
    use regex::Regex;
    
    let mut expr = latex.to_string();

    // 1. 处理分式 \frac{a}{b} -> ((a)/(b))
    let re_frac = Regex::new(r"\\frac\{([^}]+)\}\{([^}]+)\}").unwrap();
    expr = re_frac.replace_all(&expr, "(($1)/($2))").to_string();

    // 2. 处理平方根 \sqrt{a} -> sqrt(a)
    let re_sqrt = Regex::new(r"\\sqrt\{([^}]+)\}").unwrap();
    expr = re_sqrt.replace_all(&expr, "sqrt($1)").to_string();

    // 3. 处理乘号 \cdot, \times -> *
    expr = expr.replace("\\cdot", "*").replace("\\times", "*");

    // 4. 处理括号 \left( ... \right) -> ( ... )
    expr = expr.replace("\\left(", "(").replace("\\right)", ")");

    // 5. 处理特殊常数 \pi -> pi
    expr = expr.replace("\\pi", "pi");

    // 6. 清理其他 LaTeX 符号
    expr = expr.replace('{', "(").replace('}', ")");

    expr
}

/// 执行数学计算
pub fn calculate(math_expr: &str) -> Result<f64, String> {
    let processed_expr = latex_to_math_expr(math_expr);
    
    match meval::eval_str(&processed_expr) {
        Ok(result) => {
            if result.is_nan() || !result.is_finite() {
                Err("计算结果无效".to_string())
            } else {
                Ok(result)
            }
        }
        Err(e) => Err(format!("数学表达式计算错误: {}", e))
    }
}

/// 处理工具列表请求
fn handle_list_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "calculate_math",
                "description": "计算数学表达式，支持LaTeX格式",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "数学表达式，可以是LaTeX格式或普通数学表达式。例如：'\\frac{1}{2} + \\sqrt{4}' 或 '2 + 3 * 4'"
                        }
                    },
                    "required": ["expression"]
                }
            },
            {
                "name": "latex_to_expr",
                "description": "将LaTeX数学表达式转换为可计算的数学表达式",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "latex": {
                            "type": "string",
                            "description": "LaTeX数学表达式，例如：'\\frac{1}{2} + \\sqrt{4}'"
                        }
                    },
                    "required": ["latex"]
                }
            }
        ]
    })
}

/// 处理工具调用请求
fn handle_call_tool(tool_name: &str, arguments: &Value) -> Result<Value, String> {
    match tool_name {
        "calculate_math" => {
            let expression = arguments.get("expression")
                .and_then(|v| v.as_str())
                .ok_or("数学表达式不能为空")?;

            match calculate(expression) {
                Ok(result) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("表达式: {}\n结果: {}\n转换后的表达式: {}", 
                            expression, result, latex_to_math_expr(expression))
                    }]
                })),
                Err(e) => Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": format!("计算错误: {}", e)
                    }],
                    "isError": true
                }))
            }
        }
        "latex_to_expr" => {
            let latex = arguments.get("latex")
                .and_then(|v| v.as_str())
                .ok_or("LaTeX表达式不能为空")?;

            let math_expr = latex_to_math_expr(latex);
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("LaTeX: {}\n转换后的表达式: {}", latex, math_expr)
                }]
            }))
        }
        _ => Err("未知工具".to_string())
    }
}

/// 处理MCP请求
fn handle_request(request: McpRequest) -> McpResponse {
    let result: Result<Value, String> = match request.method.as_str() {
        "initialize" => {
            info!("MCP服务器初始化");
            Ok(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "mcp-math-server",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {}
                }
            }))
        }
        "tools/list" => {
            info!("列出工具");
            Ok(handle_list_tools())
        }
        "tools/call" => {
            if let Some(params) = request.params {
                let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                    Some(name) => name,
                    None => return McpResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "工具名称不能为空".to_string(),
                            data: None,
                        }),
                        id: request.id,
                    }
                };
                
                let arguments = match params.get("arguments") {
                    Some(args) => args,
                    None => return McpResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(McpError {
                            code: -32602,
                            message: "工具参数不能为空".to_string(),
                            data: None,
                        }),
                        id: request.id,
                    }
                };

                info!("调用工具: {}", tool_name);
                handle_call_tool(tool_name, arguments)
            } else {
                Err("缺少工具参数".to_string())
            }
        }
        _ => Err(format!("未知方法: {}", request.method))
    };

    match result {
        Ok(result) => McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        },
        Err(message) => McpResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(McpError {
                code: -32602,
                message,
                data: None,
            }),
            id: request.id,
        },
    }
}

/// 运行MCP服务器
pub async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("启动MCP数学服务器...");
    
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    
    for line in stdin.lock().lines() {
        let line = line?;
        
        match serde_json::from_str::<McpRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response)?;
                
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                error!("解析请求失败: {}", e);
                let error_response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("解析JSON失败: {}", e),
                        data: None,
                    }),
                    id: None,
                };
                
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }
    
    Ok(())
}