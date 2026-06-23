# IoTDB Rust Client

[![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-1.35-green.svg)](https://tokio.rs/)

Apache IoTDB 的异步 Rust 客户端。提供高性能、异步、类型安全的 IoTDB 数据库访问接口。

## ✨ 特性

- ✅ **异步/等待支持** - 基于 Tokio 的异步运行时
- ✅ **完整的 Thrift 绑定** - 自动生成所有 Thrift 协议代码
- ✅ **连接管理** - 自动连接、重连和会话管理
- ✅ **SQL 执行** - 支持执行任意 SQL 语句
- ✅ **数据插入** - 单条插入、批量插入
- ✅ **数据查询** - 最新数据、时间范围、聚合查询
- ✅ **元数据管理** - 创建/查询/删除存储组、时间序列
- ✅ **系统信息** - 版本查询、系统状态
- ✅ **错误处理** - 完善的错误类型和处理机制
- ✅ **超时控制** - 可配置的连接和操作超时
- ✅ **类型安全** - 利用 Rust 的强类型系统

## 📋 目录

- [安装](#安装)
- [快速开始](#快速开始)
- [示例](#示例)
- [API 文档](#api-文档)
- [性能测试](#性能测试)
- [开发指南](#开发指南)
- [许可证](#许可证)

## 🚀 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
iotdb-rust-client = { path = "./iotdb-client/rust" }
# 或者当发布到 crates.io 后：
# iotdb-rust-client = "0.1.0"

# 推荐同时添加以下依赖
tokio = { version = "1.35", features = ["full"] }
serde_json = "1.0"
chrono = "0.4"
```

## 🎯 快速开始
### 基础连接
```rust
use iotdb_rust_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
// 创建客户端
let client = Client::new("localhost", 6667);

// 连接
client.connect().await?;

// 查询数据
let result = client.query("SHOW DATABASES").await?;
println!("{:?}", result);

// 断开连接
client.disconnect().await?;

Ok(())
}
```
### 插入数据
```rust
use iotdb_rust_client::{Client, InsertRecord};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("localhost", 6667);
    client.connect().await?;
    
    // 单条插入
    let timestamp = chrono::Utc::now().timestamp_millis();
    client.insert(
        "root.sg1.d1.temperature",
        timestamp,
        json!(36.5)
    ).await?;
    
    // 批量插入
    let records = vec![
        InsertRecord {
            path: "root.sg1.d1.temperature".to_string(),
            timestamp: timestamp + 1000,
            value: json!(37.2),
        },
        InsertRecord {
            path: "root.sg1.d1.humidity".to_string(),
            timestamp: timestamp + 1000,
            value: json!(68.5),
        },
    ];
    
    client.batch_insert(&records).await?;
    
    client.disconnect().await?;
    Ok(())
}
```
### 使用认证
```rust
use iotdb_rust_client::{Client, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::new("localhost", 6667)
        .with_auth("root", "root")
        .with_timeout(60);
    
    let client = Client::with_config(config);
    client.connect().await?;
    
    // 执行操作...
    
    Ok(())
}
```
## 📚 示例
### 示例列表

示例	说明	运行命令
connect.rs	连接和断开测试	cargo run --example connect
insert.rs	数据插入演示	cargo run --example insert
query.rs	数据查询演示	cargo run --example query
batch_insert.rs	批量插入和性能测试	cargo run --example batch_insert
metadata.rs	元数据操作演示	cargo run --example metadata
error_handling.rs	错误处理演示	cargo run --example error_handling
timeout.rs	超时处理演示	cargo run --example timeout
config.rs	配置信息展示	cargo run --example config

### 运行示例
```bash
# 克隆项目
git clone https://github.com/apache/iotdb.git
cd iotdb/iotdb-client/rust

# 确保 IoTDB 服务正在运行
# 默认连接 localhost:6667

# 运行连接测试
cargo run --example connect

# 运行插入示例
cargo run --example insert

# 运行查询示例
cargo run --example query

# 运行批量插入（性能测试）
cargo run --example batch_insert
```

### 示例输出
连接示例输出：
```text
╔════════════════════════════════════════════╗
║     IoTDB Rust Client - Connection Test    ║
╚════════════════════════════════════════════╝

✅ Client created: localhost:6667

--- Before Connection ---
Connected: false

--- Connecting ---
✅ Connected to IoTDB at localhost:6667
Connected: true

--- Disconnecting ---
✅ Disconnected from IoTDB
Connected: false

✅ Connection test completed successfully!
```

## 📖 API 文档
### 核心类型
#### Client  主要的客户端类型，用于所有数据库操作。
```rust
impl Client {
    // 创建客户端
    pub fn new(host: &str, port: u16) -> Self;
    pub fn with_config(config: ClientConfig) -> Self;

    // 连接管理
    pub async fn connect(&self) -> Result<()>;
    pub async fn disconnect(&self) -> Result<()>;
    pub async fn is_connected(&self) -> bool;

    // SQL 执行
    pub async fn execute(&self, sql: &str) -> Result<QueryResult>;
    pub async fn query(&self, sql: &str) -> Result<QueryResult>;
    pub async fn update(&self, sql: &str) -> Result<u64>;

    // 数据操作
    pub async fn insert(&self, path: &str, timestamp: i64, value: Value) -> Result<()>;
    pub async fn batch_insert(&self, records: &[InsertRecord]) -> Result<()>;
    pub async fn delete(&self, path: &str, timestamp: i64) -> Result<()>;
    pub async fn delete_series(&self, path: &str) -> Result<()>;

    // 查询
    pub async fn query_latest(&self, path: &str, limit: usize) -> Result<QueryResult>;
    pub async fn query_range(&self, path: &str, start_time: i64, end_time: i64) -> Result<QueryResult>;
    pub async fn query_aggregate(&self, path: &str, aggregate: &str, start_time: i64, end_time: i64) -> Result<QueryResult>;

    // 元数据
    pub async fn show_databases(&self) -> Result<QueryResult>;
    pub async fn show_timeseries(&self, path: Option<&str>) -> Result<QueryResult>;
    pub async fn show_storage_groups(&self) -> Result<QueryResult>;
    pub async fn show_devices(&self) -> Result<QueryResult>;
    pub async fn show_child_paths(&self, path: &str) -> Result<QueryResult>;
    pub async fn show_data_nodes(&self) -> Result<QueryResult>;
    pub async fn show_config_nodes(&self) -> Result<QueryResult>;

    // 系统
    pub async fn system_info(&self) -> Result<QueryResult>;
    pub async fn version(&self) -> Result<String>;
}
```
#### ClientConfig-客户端配置。
```rust
pub struct ClientConfig {
    pub host: String,              // 主机地址
    pub port: u16,                 // 端口
    pub timeout_secs: u64,         // 连接超时（秒）
    pub session_timeout_secs: u64, // 会话超时（秒）
    pub username: Option<String>,  // 用户名
    pub password: Option<String>,  // 密码
    pub enable_tls: bool,          // 是否启用 TLS
}

impl ClientConfig {
    pub fn new(host: &str, port: u16) -> Self;
    pub fn with_auth(mut self, username: &str, password: &str) -> Self;
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self;
    pub fn with_session_timeout(mut self, timeout_secs: u64) -> Self;
}
```

####  QueryResult-查询结果。
```rust
pub struct QueryResult {
    pub columns: Vec<String>,       // 列名
    pub rows: Vec<Row>,             // 数据行
    pub affected_rows: Option<u64>, // 影响的行数
}
```

#### InsertRecord-插入记录。
```rust
pub struct InsertRecord {
    pub path: String,              // 时间序列路径
    pub timestamp: i64,            // 时间戳（毫秒）
    pub value: serde_json::Value,  // 值
}
```
#### 错误类型
```rust
pub enum Error {
    Connection(String),      // 连接错误
    Thrift(thrift::Error),   // Thrift 错误
    Io(std::io::Error),      // IO 错误
    Timeout(String),         // 超时错误
    Auth(String),            // 认证错误
    InvalidParameter(String), // 无效参数
    Execution(String),       // 执行错误
    Serialization(String),   // 序列化错误
    NotImplemented(String),  // 未实现
}
```

## ⚡ 性能测试
### 批量插入性能
在 examples/batch_insert.rs 中提供了性能测试示例：
```bash
# 测试 10,000 条记录的批量插入性能
cargo run --example batch_insert
```

### 典型性能结果（基于本地测试）：
- 10,000 条记录
- 批量大小：500 条/批
- 插入速度：~5,000-8,000 条/秒

## 🔧 开发指南
### 构建项目
```bash
# 克隆项目
git clone https://github.com/apache/iotdb.git
cd iotdb/iotdb-client/rust

# 构建
cargo build

# 构建发布版本
cargo build --release

# 运行测试
cargo test

# 生成文档
cargo doc --open
```
### 代码生成
Thrift 代码在 build.rs 中自动生成，无需手动操作。

### 项目结构
```text
iotdb-client/rust/
├── Cargo.toml              # 项目配置
├── build.rs                # Thrift 代码生成
├── src/
│   ├── lib.rs              # 主库代码
│   └── thrift/             # 生成的 Thrift 代码
│       ├── mod.rs
│       ├── common.rs
│       ├── client.rs
│       ├── datanode.rs
│       └── confignode.rs
├── examples/               # 示例代码
│   ├── config.rs
│   ├── connect.rs
│   ├── insert.rs
│   ├── query.rs
│   ├── batch_insert.rs
│   ├── metadata.rs
│   ├── error_handling.rs
│   └── timeout.rs
└── target/                 # 构建输出
```

### 添加新功能
1. 在 src/lib.rs 中添加新方法
2. 在 examples/ 中添加示例
3. 更新文档
4. 运行测试

### 版本管理
#### 项目使用语义化版本控制：
- 0.1.x - 开发版本，API 可能变化
- 1.x.x - 稳定版本，API 稳定

## 🤝 贡献
欢迎贡献！请遵循以下步骤：
- Fork 项目
- 创建特性分支 (git checkout -b feature/amazing-feature)
- 提交代码 (git commit -m 'Add some amazing feature')
- 推送到分支 (git push origin feature/amazing-feature)
- 创建 Pull Request

### 贡献指南
- 遵循 Rust 代码风格 (rustfmt)
- 添加适当的测试
- 更新文档
- 保持向后兼容性

## 📝 许可证
该项目使用 Apache License 2.0 许可证。
```text
Copyright 2024 Apache IoTDB

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## 📞 联系我们
[Apache IoTDB 官网](https://iotdb.apache.org/)
[GitHub Issues](https://github.com/apache/iotdb/issues)

## 🙏 致谢
- [Apache Thrift](https://thrift.apache.org/) - RPC 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [Apache IoTDB](https://iotdb.apache.org/) - 时序数据库

> 注意: 本项目正在积极开发中，API 可能会有变化。建议使用最新的稳定版本。

