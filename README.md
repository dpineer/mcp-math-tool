# MCP 数学工具

一个支持 LaTeX 格式数学表达式计算的工具，提供三种集成方式：
1. **MCP 协议**：与 Claude Desktop 等 AI 客户端集成
2. **HTTP API**：与 Dify 等 Web 平台集成  
3. **混合模式**：同时运行HTTP服务和MCP接口


## 功能特性

- 支持 LaTeX 格式的数学表达式
- 支持分式：`\frac{a}{b}`
- 支持平方根：`\sqrt{a}`
- 支持数学常数：`\pi`
- 支持基本运算符：`+`, `-`, `*`, `/`
- 双模式支持：MCP Stdio 协议和 HTTP REST API

## 安装与编译

### 前提条件
- 安装 Rust 工具链 (rustc, cargo)

### 编译项目
```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/mcp-math-tool`

## 使用方式一：MCP 协议（Claude Desktop）

### 配置 Claude Desktop

编辑 Claude Desktop 的配置文件（位置因操作系统而异）：

#### macOS
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

#### Windows
```
%APPDATA%\Claude\claude_desktop_config.json
```

#### Linux
```
~/.config/Claude/claude_desktop_config.json
```

在配置文件中添加以下内容：

```json
{
  "mcpServers": {
    "rust-math": {
      "command": "/绝对路径/到/mcp-math-tool/target/release/mcp-math-tool",
      "args": []
    }
  }
}
```

请将 `/绝对路径/到/mcp-math-tool` 替换为实际的绝对路径。

### 使用方法

在 Claude Desktop 中，你可以这样提问：

- "请使用数学工具计算这个算式：`\frac{50}{2} + \sqrt{16} \cdot 2`"
- "计算 `\frac{1}{2} + \frac{3}{4}`"
- "计算 `2 \cdot \pi`"

## 使用方式二：HTTP API（Dify 集成）

### 启动 HTTP 服务器

```bash
./target/release/mcp-math-tool
```

服务器将在 `http://localhost:3000` 启动。

## 使用方式三：混合模式（MCP + HTTP API）

### 架构概述

混合模式允许同时运行两种服务：
- **Rust HTTP服务**：在端口3000提供REST API（如原有功能）
- **Node.js MCP服务**：通过MCP协议提供数学计算工具

### 启动步骤

1. **启动Rust HTTP服务**（端口3000）：
```bash
cargo run --release
```

2. **启动Node.js MCP服务**（独立进程）：
```bash
cd /home/dpiner/文档/Cline/MCP/mcp-math-server
npm run build
node build/index.js
```

### VS Code集成配置

MCP服务器已配置在VS Code中，提供以下工具：
- `calculate_math`: 计算数学表达式，支持LaTeX格式
- `latex_to_expr`: 将LaTeX转换为可计算表达式

### 使用示例

**通过MCP接口计算：**
```
计算: \frac{1}{2} + \sqrt{4}
结果: 2.5
```

**通过HTTP API计算：**
```bash
curl -X POST http://localhost:3000/calculate \
  -H "Content-Type: application/json" \
  -d '{"expression": "\\frac{1}{2} + \\sqrt{4}"}'
```

### 混合模式优势

- **灵活性**：支持多种集成方式
- **兼容性**：同时满足AI客户端和Web平台需求
- **可扩展性**：可独立升级任一服务
- **容错性**：一个服务故障不影响另一个
+++++++
```

### API 接口

#### 1. 健康检查
```bash
curl -X POST http://localhost:3000/health -H "Content-Type: application/json"
```

响应：
```json
{"status":"ok","version":"1.0.0","service":"mcp-math-api"}
```

#### 2. 计算表达式
```bash
curl -X POST http://localhost:3000/calculate \
  -H "Content-Type: application/json" \
  -d '{"expression": "\\frac{50}{2} + \\sqrt{16} \\cdot 2"}'
```

成功响应：
```json
{"result":33.0,"expression":"\\frac{50}{2} + \\sqrt{16} \\cdot 2","success":true,"error":null}
```

错误响应：
```json
{"result":0.0,"expression":"invalid expression","success":false,"error":"计算错误: Parse error: Unexpected token at byte 8."}
```

### 在 Dify 中配置

1. **部署服务**：将程序部署到服务器（或使用 ngrok/Cloudflare Tunnel 暴露本地端口）

2. **创建 OpenAPI 文档**：项目已包含 `openapi.yaml` 文件

3. **在 Dify 中添加自定义工具**：
   - 进入 Dify 控制台 → 工具 (Tools) → 自定义 (Custom) → 创建自定义工具
   - 上传或粘贴 `openapi.yaml` 内容
   - 填写服务器基础 URL（例如 `http://localhost:3000` 或你的公网地址）
   - Dify 会自动解析出 `calculateLatex` 工具

4. **在工作流中使用**：在 Dify 工作流或 Agent 中添加并使用数学计算工具

## 支持的 LaTeX 语法

| LaTeX 语法 | 转换后 | 示例 |
|------------|--------|------|
| `\frac{a}{b}` | `((a)/(b))` | `\frac{1}{2}` → `((1)/(2))` |
| `\sqrt{a}` | `sqrt(a)` | `\sqrt{16}` → `sqrt(16)` |
| `\pi` | `pi` | `2 \cdot \pi` → `2 * pi` |
| `\cdot`, `\times` | `*` | `2 \cdot 3` → `2 * 3` |
| `\left(`, `\right)` | `(`, `)` | `\left( x + y \right)` → `( x + y )` |

## 项目结构

```
mcp-math-tool/
├── Cargo.toml          # 项目配置和依赖
├── src/
│   └── main.rs         # 主程序代码（HTTP API + 数学逻辑）
├── openapi.yaml        # OpenAPI 3.0 规范文档
└── README.md           # 本文件
```

## 技术细节

### 依赖库
- `tokio`: 异步运行时
- `axum`: HTTP Web 框架
- `tower-http`: HTTP 中间件
- `serde` / `serde_json`: JSON 序列化
- `regex`: 正则表达式处理
- `meval`: 数学表达式求值
- `anyhow`: 错误处理
- `tracing`: 日志记录

### 数学处理流程
1. 接收 LaTeX 字符串
2. 使用正则表达式转换语法
3. 使用 `meval` 库计算表达式
4. 返回计算结果

### HTTP API 特性
- RESTful 设计
- 完整的错误处理
- CORS 支持
- 结构化日志记录
- 健康检查端点

## 测试示例

### MCP 协议测试
```bash
# 测试初始化
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test-client","version":"1.0.0"}},"id":1}' | ./target/release/mcp-math-tool

# 测试计算
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"calculate_latex","arguments":{"expression":"\\frac{50}{2} + \\sqrt{16} \\cdot 2"}},"id":3}' | ./target/release/mcp-math-tool
```

### HTTP API 测试
```bash
# 健康检查
curl -X POST http://localhost:3000/health -H "Content-Type: application/json"

# 计算测试
curl -X POST http://localhost:3000/calculate \
  -H "Content-Type: application/json" \
  -d '{"expression": "\\frac{50}{2} + \\sqrt{16} \\cdot 2"}'

# 复杂表达式测试
curl -X POST http://localhost:3000/calculate \
  -H "Content-Type: application/json" \
  -d '{"expression": "\\left( \\frac{3}{4} + \\frac{5}{6} \\right) \\cdot \\sqrt{25} - \\frac{\\pi}{2}"}'
```

## 扩展功能

如需扩展支持更多数学函数（如三角函数、对数等），可以修改 `latex_to_math_expr` 函数，添加相应的正则表达式转换规则。

## 部署建议

### 本地开发
```bash
./target/release/mcp-math-tool
```

### 生产部署
1. 使用 systemd 或 supervisor 管理进程
2. 配置反向代理（Nginx/Apache）
3. 启用 HTTPS
4. 设置适当的防火墙规则

### 使用 ngrok 快速暴露本地服务
```bash
ngrok http 3000
```

## 许可证

MIT
