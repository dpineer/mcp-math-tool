# MCP 数学工具

一个支持HTTP API和MCP协议的数学计算工具，能够处理LaTeX格式的数学表达式。

## 功能特性

- **HTTP API**: 提供RESTful API接口进行数学计算
- **MCP协议**: 支持Model Context Protocol
- **LaTeX支持**: 支持LaTeX格式的数学表达式
- **Docker部署**: 容器化部署，易于使用

## HTTP API 端点

### `/calculate` - 数学计算
- **方法**: POST
- **请求体**:
```json
{
  "expression": "数学表达式"
}
```

- **示例**:
```bash
curl -X POST http://localhost:3000/calculate \
  -H "Content-Type: application/json" \
  -d '{"expression": "2 + 3 * 4"}'
```

- **响应**:
```json
{
  "result": 14.0,
  "expression": "2 + 3 * 4",
  "success": true,
  "error": null
}
```

### `/health` - 健康检查
- **方法**: POST
- **响应**:
```json
{
  "status": "ok",
  "version": "1.0.0",
  "service": "mcp-math-api"
}
```

## Docker 部署

### 构建镜像
```bash
docker build -t mcp-math-tool .
```

### 运行容器
```bash
# HTTP服务器模式（默认）
docker run -p 3000:3000 mcp-math-tool

# 或者指定端口
docker run -p 8080:8080 mcp-math-tool http --addr 0.0.0.0:8080
```

### Docker Compose
```yaml
version: '3.8'
services:
  mcp-math-tool:
    build: .
    ports:
      - "3000:3000"
    restart: unless-stopped
```

## 支持的表达式格式

- 普通数学表达式: `2 + 3 * 4`
- LaTeX格式:
  - 分式: `\frac{1}{2}`
  - 平方根: `\sqrt{4}`
  - 乘号: `\cdot`, `\times`
  - 括号: `\left(`, `\right)`
  - 常数: `\pi`

## MCP协议模式

运行MCP服务器模式:
```bash
docker run mcp-math-tool mcp
```

## 环境要求

- Docker 20.10+
- Rust 1.84+ (如果本地构建)

## 许可证

MIT License