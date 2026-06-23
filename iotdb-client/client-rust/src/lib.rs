//! Apache IoTDB Rust Client
#![allow(warnings)]

// ============================================================================
// Thrift 生成的代码
// ============================================================================

pub mod generated {
    include!("thrift/mod.rs");
}

pub use generated::*;

// ============================================================================
// 外部依赖
// ============================================================================

use ::thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol};
use ::thrift::transport::TTcpChannel;

use std::sync::Arc;
use std::time::Duration;
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use tokio::sync::Mutex;
use crate::client::{TIClientRPCServiceSyncClient, TSOpenSessionReq, TSProtocolVersion};
// ============================================================================
// 常量
// ============================================================================

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_PORT: u16 = 6667;

//字典

// ============================================================================
// 错误类型
// ============================================================================

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Thrift error: {0}")]
    Thrift(#[from] ::thrift::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, Error>;

use thrift::transport::{ReadHalf, TFramedReadTransport, TFramedWriteTransport, WriteHalf};
// 🔥 1. 确保引入了 Compact 协议
use thrift::protocol::{TCompactInputProtocol, TCompactOutputProtocol};
use log::{error, warn};

// 🔥 2. 将此处的 TBinary 全部替换为 TCompact
// 恢复顶部的类型别名
pub type ThriftClient = client::IClientRPCServiceSyncClient<
    TBinaryInputProtocol<TFramedReadTransport<ReadHalf<TTcpChannel>>>,
    TBinaryOutputProtocol<TFramedWriteTransport<WriteHalf<TTcpChannel>>>,
>;

// ============================================================================
// 数据类型
// ============================================================================

#[derive(Debug, Clone)]
pub struct InsertRecord {
    pub path: String,
    pub timestamp: i64,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Row {
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub affected_rows: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TableRowInsert {
    pub table_name: String,                         // 表名（如 factory_pipeline）
    pub time: i64,                                  // TIMESTAMP 核心时间戳（根据数据库设置可以是 ms/us/ns）
    pub tags: HashMap<String, String>,              // 标签列（用于设备分型定位，如 device_id: "d1", area: "zone_a"）
    pub fields: HashMap<String, serde_json::Value>, // 物理测点列（如 temperature: 23.5, status: true）
}

// ============================================================================
// 客户端配置
// ============================================================================

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enable_tls: bool,
    pub session_timeout_secs: u64,
    pub sql_dialect: String,
    pub database: Option<String>,
    pub zone_id: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: DEFAULT_PORT,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            username: Some("root".to_string()),
            password: Some("root".to_string()),
            enable_tls: false,
            session_timeout_secs: 3600,
            sql_dialect: "TABLE".to_string(),
            database: None,
            zone_id: "Asia/Shanghai".to_string(), // 🚀 2. 默认国内标准时区
        }
    }
}


impl ClientConfig {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            ..Default::default()
        }
    }


    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_session_timeout(mut self, timeout_secs: u64) -> Self {
        self.session_timeout_secs = timeout_secs;
        self
    }


    pub fn with_sql_dialect(mut self, sql_dialect: &str) -> Self {
        self.sql_dialect = sql_dialect.to_string();
        self
    }

    pub fn with_zone_id(mut self, zone_id: &str) -> Self {
        self.zone_id = zone_id.to_string();
        self
    }
    pub fn with_server_addr(mut self, host: &str,port: u16) -> Self {
        self.host = host.to_string();
        self.port = port;
        self
    }

}

// ============================================================================
// IoTDB 客户端
// ============================================================================

pub struct Client {
    pub config: ClientConfig,
    session_id: Arc<Mutex<Option<i64>>>,
    connected: Arc<Mutex<bool>>,
    thrift_client: Arc<Mutex<Option<ThriftClient>>>,
    last_active: Arc<Mutex<Option<std::time::Instant>>>,
    sequence: Arc<Mutex<i64>>,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Self {
        Self::with_config(ClientConfig::new(host, port))
    }

    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            config,
            session_id: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            thrift_client: Arc::new(Mutex::new(None)),
            last_active: Arc::new(Mutex::new(None)),
            sequence: Arc::new(Mutex::new(0)),
        }
    }

    // ========================================================================
    // 连接管理
    // ========================================================================
    pub async fn connect(&self) -> Result<()> {
        if self.is_connected().await {
            self.disconnect().await?;
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        println!("Connecting to {}...", addr);

        let mut channel = TTcpChannel::new();

        match channel.open(&addr) {
            Ok(()) => {
                println!("Connected to {}:{}", self.config.host, self.config.port);

                use thrift::transport::TIoChannel;
                let (i_chan, o_chan) = channel.split()
                    .map_err(|e| Error::Connection(format!("Failed to split channel: {}", e)))?;

                // 🔥 核心修改：使用 TFramed 包装裸 TCP 管道
                let i_tran = TFramedReadTransport::new(i_chan);
                let o_tran = TFramedWriteTransport::new(o_chan);

                // 将包装后的传输层喂给协议层
                let input_protocol = TBinaryInputProtocol::new(i_tran, true);
                let output_protocol = TBinaryOutputProtocol::new(o_tran, true);

                use crate::generated::client::IClientRPCServiceSyncClient;
                let client = IClientRPCServiceSyncClient::new(input_protocol, output_protocol);
                // 🔥 修复点 1：用独立大括号 {} 限制锁的范围，让其执行完立刻 Drop 释放锁
                {
                    let mut connected_guard = self.connected.lock().await;
                    *connected_guard = true;
                }

                {
                    let mut client_guard = self.thrift_client.lock().await;
                    *client_guard = Some(client);
                } // 👈 核心：client_guard 在这里被自动销毁，自锁解除！

                {
                    let mut last_active_guard = self.last_active.lock().await;
                    *last_active_guard = Some(std::time::Instant::now());
                }

                // 🔥 修复点 2：解决方式1的 "No active session" 报错
                // IoTDB 哪怕查版本也需要 Session。若配置未填，默认采用 "root" / "root" 进行底层的 open_session
                let username = self.config.username.as_deref().unwrap_or("root");
                let password = self.config.password.as_deref().unwrap_or("root");

                // 此时 thrift_client 的锁是解开状态，调用 authenticate 绝不会死锁
                self.authenticate(username, password).await?;

                Ok(())
            }
            Err(e) => Err(Error::Connection(format!("Failed to connect: {}", e))),
        }
    }

    pub async fn disconnect(&self) -> Result<()> {
        let mut connected_guard = self.connected.lock().await;
        *connected_guard = false;

        let mut channel_guard = self.thrift_client.lock().await;
        if let Some(channel) = channel_guard.take() {
            drop(channel);
        }

        let mut session_guard = self.session_id.lock().await;
        *session_guard = None;

        let mut last_active_guard = self.last_active.lock().await;
        *last_active_guard = None;

        println!("Disconnected from {}:{}", self.config.host, self.config.port);
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        let connected_guard = self.connected.lock().await;
        *connected_guard
    }

    pub async fn get_session_id(&self) -> Option<i64> {
        let session_guard = self.session_id.lock().await;
        *session_guard
    }
    // ========================================================================
    // 增加状态拦截的认证实现（已修复配置传递，适配工业级动态配置）
    // ========================================================================
    async fn authenticate(&self, username: &str, password: &str) -> Result<()> {
        let username_owned = username.to_string();
        let password_owned = password.to_string();
        let thrift_client = self.thrift_client.clone();

        // 🚀 动态读取 ClientConfig 的配置，拒绝硬编码
        let sql_dialect = self.config.sql_dialect.clone();
        let database = self.config.database.clone();
        let zone_id_owned = self.config.zone_id.clone();

        let session_id = tokio::task::spawn_blocking(move || {
            use crate::generated::client::TSOpenSessionReq;
            use crate::generated::client::TSProtocolVersion;

            let mut client_guard = thrift_client.blocking_lock();
            let client = client_guard.as_mut()
                .ok_or_else(|| Error::Connection("无激活的 Thrift 客户端".to_string()))?;

            // 🚀 工业级动态组装：根据用户配置注入 SQL 方言和默认 Database
            let mut configuration_map = BTreeMap::new();
            configuration_map.insert("sql_dialect".to_string(), sql_dialect);
            if let Some(db) = database {
                configuration_map.insert("db".to_string(), db);
            }

            let req = TSOpenSessionReq {
                client_protocol: TSProtocolVersion::IOTDB_SERVICE_PROTOCOL_V3,
                zone_id: zone_id_owned,
                username: username_owned,
                password: Some(password_owned),
                configuration: Some(configuration_map),
            };

            let resp = client.open_session(req).map_err(Error::Thrift)?;

            if resp.status.code != 200 && resp.status.code != 0 {
                return Err(Error::Execution("IoTDB 服务端拒绝登录: ...".to_string()));
            }
            Ok::<i64, Error>(resp.session_id.unwrap_or_default())
        }).await
            .map_err(|e| Error::Connection(format!("RPC 线程池调度失败: {}", e)))??;

        let mut session_guard = self.session_id.lock().await;
        *session_guard = Some(session_id);
        Ok(())
    }
    // ========================================================================
    // SQL 执行接口（真实 Thrift RPC 实现版）
    // ========================================================================
    pub async fn execute(&self, sql: &str) -> Result<QueryResult> {
        let max_retries = 3; // 允许断线重试 3 次
        let mut retry_count = 0;

        loop {
            // 🚀 1. 健康状态自愈：未连接或 Session 丢失时，触发自动重连
            if !self.is_connected().await || self.get_session_id().await.is_none() {
                warn!("检测到连接未建立或断开，触发工业自愈，正在尝试自动重连...");
                if let Err(e) = self.connect().await {
                    error!("自动重连失败: {}", e);
                    if retry_count >= max_retries {
                        return Err(Error::Connection(format!("重连超出最大次数: {}", e)));
                    }
                    // 指数退避策略：1s, 2s, 4s... 避免瞬时网络抖动时高频冲击服务器
                    tokio::time::sleep(Duration::from_secs(1 << retry_count)).await;
                    retry_count += 1;
                    continue;
                }
            }

            let session_id = self.get_session_id().await
                .ok_or_else(|| Error::Connection("未分配有效 Session".to_string()))?;
            let sql_str = sql.to_string();
            let thrift_client = self.thrift_client.clone();

            // 2. 投递到线程池执行核心 Thrift RPC 同步阻塞逻辑
            let res = tokio::task::spawn_blocking(move || {
                use crate::generated::client::TSExecuteStatementReq;
                use byteorder::{BigEndian, ReadBytesExt};
                use std::io::Cursor;

                let mut client_guard = thrift_client.blocking_lock();
                let client = client_guard.as_mut()
                    .ok_or_else(|| Error::Connection("无激活的 Thrift 客户端".to_string()))?;

                // 向服务端申请 Statement ID
                let statement_id = client.request_statement_id(session_id)?;

                let req = TSExecuteStatementReq {
                    session_id,
                    statement: sql_str,
                    statement_id,
                    fetch_size: Some(4096), // 🚀 工业吞吐优化：增大单批次拉取量，降低高频数据交互时的网络 RTT 损耗
                    timeout: Some(30000),
                    enable_redirect_query: Some(true),
                    jdbc_query: Some(false),
                };

                let resp = client.execute_statement(req).map_err(Error::Thrift)?;

                // 拦截服务端内部产生的业务异常
                if resp.status.code != 200 && resp.status.code != 0 {
                    return Err(Error::Execution(format!(
                        "IoTDB 执行 SQL 失败 (Code {}): {}",
                        resp.status.code,
                        resp.status.message.unwrap_or_default()
                    )));
                }

                // 🚀 3. 【编译修复】在闭包内部正确提取元数据和初始化数据集
                let columns = resp.columns.unwrap_or_default();
                let data_types = resp.data_type_list.unwrap_or_default();
                let mut rows = Vec::new();

                // 4. 🔥 兼容分支 A：处理传统树模型的 query_data_set
                if let Some(dataset) = resp.query_data_set {
                    let mut cursors: Vec<Cursor<Vec<u8>>> = dataset.value_list.into_iter().map(Cursor::new).collect();

                    if !cursors.is_empty() {
                        while cursors[0].position() < cursors[0].get_ref().len() as u64 {
                            let mut fields = std::collections::HashMap::new();

                            for (col_idx, cursor) in cursors.iter_mut().enumerate() {
                                let col_name = match columns.get(col_idx) {
                                    Some(name) => name.clone(),
                                    None => continue,
                                };

                                let data_type = data_types.get(col_idx).map(|s| s.as_str()).unwrap_or("TEXT");

                                let value = match data_type {
                                    "TEXT" | "STRING" => {
                                        if let Ok(str_len) = cursor.read_i32::<BigEndian>() {
                                            let mut buf = vec![0u8; str_len as usize];
                                            if std::io::Read::read_exact(cursor, &mut buf).is_ok() {
                                                String::from_utf8(buf).ok().map(serde_json::Value::String)
                                            } else { None }
                                        } else { None }
                                    }
                                    "INT32" => {
                                        cursor.read_i32::<BigEndian>().ok().map(|v| serde_json::Value::Number(v.into()))
                                    }
                                    "INT64" | "TIMESTAMP" => {
                                        cursor.read_i64::<BigEndian>().ok().map(|v| serde_json::Value::Number(v.into()))
                                    }
                                    "FLOAT" => {
                                        cursor.read_f32::<BigEndian>().ok()
                                            .and_then(|v| serde_json::Number::from_f64(v as f64).map(serde_json::Value::Number))
                                    }
                                    "DOUBLE" => {
                                        cursor.read_f64::<BigEndian>().ok()
                                            .and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number))
                                    }
                                    "BOOLEAN" => {
                                        cursor.read_u8().ok().map(|v| serde_json::Value::Bool(v != 0))
                                    }
                                    _ => None,
                                };

                                if let Some(val) = value {
                                    fields.insert(col_name, val);
                                }
                            }
                            rows.push(Row { fields });
                        }
                    }
                }
                // 4. 🔥 兼容分支 B：处理关系型 TABLE 模型的 query_result (TsBlock 工业重构版)
                else if let Some(blocks) = resp.query_result {
                    for block_bytes in blocks {
                        if block_bytes.is_empty() { continue; }
                        let mut cursor = Cursor::new(block_bytes);

                        if let (Ok(position_count), Ok(column_count)) = (
                            cursor.read_i32::<BigEndian>(),
                            cursor.read_i32::<BigEndian>()
                        ) {
                            // 预先初始化当前数据块（Block）的行容器
                            let mut block_rows = vec![std::collections::HashMap::new(); position_count as usize];

                            // TsBlock 在底层是列式存储的，必须外层循环遍历列，内层循环遍历行
                            for col_idx in 0..column_count {
                                let col_name = match columns.get(col_idx as usize) {
                                    Some(name) => name.clone(),
                                    None => continue,
                                };
                                let data_type = data_types.get(col_idx as usize).map(|s| s.as_str()).unwrap_or("TEXT");

                                // 解析当前列的 Null 位图状态
                                let _maybe_null = cursor.read_u8().unwrap_or(0);
                                let mut null_bitmap = vec![0u8; ((position_count + 7) / 8) as usize];
                                if _maybe_null > 0 {
                                    let _ = std::io::Read::read_exact(&mut cursor, &mut null_bitmap);
                                }

                                for row_idx in 0..position_count {
                                    let is_null = _maybe_null > 0 &&
                                        (null_bitmap[(row_idx / 8) as usize] & (1 << (row_idx % 8))) == 0;

                                    if !is_null {
                                        // 🚀 工业级漏洞修复：根据该列的工业真实类型（如 FLOAT/INT）精准切割字节
                                        let value = match data_type {
                                            "TEXT" | "STRING" => {
                                                if let Ok(str_len) = cursor.read_i32::<BigEndian>() {
                                                    let mut buf = vec![0u8; str_len as usize];
                                                    if std::io::Read::read_exact(&mut cursor, &mut buf).is_ok() {
                                                        String::from_utf8(buf).ok().map(serde_json::Value::String)
                                                    } else { None }
                                                } else { None }
                                            }
                                            "INT32" => {
                                                cursor.read_i32::<BigEndian>().ok().map(|v| serde_json::Value::Number(v.into()))
                                            }
                                            "INT64" | "TIMESTAMP" => {
                                                cursor.read_i64::<BigEndian>().ok().map(|v| serde_json::Value::Number(v.into()))
                                            }
                                            "FLOAT" => {
                                                cursor.read_f32::<BigEndian>().ok()
                                                    .and_then(|v| serde_json::Number::from_f64(v as f64).map(serde_json::Value::Number))
                                            }
                                            "DOUBLE" => {
                                                cursor.read_f64::<BigEndian>().ok()
                                                    .and_then(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number))
                                            }
                                            "BOOLEAN" => {
                                                cursor.read_u8().ok().map(|v| serde_json::Value::Bool(v != 0))
                                            }
                                            _ => None,
                                        };

                                        if let Some(val) = value {
                                            block_rows[row_idx as usize].insert(col_name.clone(), val);
                                        }
                                    }
                                }
                            }

                            // 将当前 Block 解析出的所有行追加到总结果集中
                            for fields in block_rows {
                                rows.push(Row { fields });
                            }
                        }
                    }
                }

                // 🚀 5. 【编译修复】正确构件并返回最终的工业级通用查询实体
                Ok::<QueryResult, Error>(QueryResult {
                    columns,
                    rows,
                    affected_rows: None,
                })
            }).await;

            // 🚀 5. 传输层容错判定拦截
            match res {
                Ok(Ok(query_result)) => {
                    if let Some(mut last_active_guard) = self.last_active.lock().await.as_mut() {
                        *last_active_guard = std::time::Instant::now();
                    }
                    return Ok(query_result);
                }
                Ok(Err(Error::Thrift(thrift_err))) => {
                    warn!("捕获到底层传输异常 (Thrift Error): {:?}. 正在清理失效连接，准备进入下一轮自愈...", thrift_err);
                    self.disconnect().await?; // 立即清理本地残留状态
                    if retry_count >= max_retries {
                        return Err(Error::Thrift(thrift_err));
                    }
                    retry_count += 1;
                    tokio::time::sleep(Duration::from_millis(500)).await; // 闪断等待后循环，下一次循环将自动触发 connect()
                }
                Ok(Err(biz_err)) => {
                    // 如果是 SQL 语法错误或表不存在等业务异常，重试也无意义，直接抛给上层工业逻辑
                    return Err(biz_err);
                }
                Err(spawn_err) => {
                    return Err(Error::Connection(format!("RPC 线程池调度崩溃: {}", spawn_err)));
                }
            }
        }
    }


    pub async fn query(&self, sql: &str) -> Result<QueryResult> {
        self.execute(sql).await
    }

    pub async fn update(&self, sql: &str) -> Result<u64> {
        let result = self.execute(sql).await?;
        Ok(result.affected_rows.unwrap_or(0))
    }

    pub async fn insert(
        &self,
        device_path: &str,
        measurement: &str,
        timestamp: i64,
        value: impl Into<serde_json::Value>
    ) -> Result<()> {
        let value_json = value.into();
        let val_str = match &value_json {
            serde_json::Value::String(s) => format!("'{}'", s),
            _ => value_json.to_string(),
        };
        // 🚀 将设备与测点分离，时间列使用标准 `time`
        let sql = format!(
            "INSERT INTO {} (time, {}) VALUES ({}, {})",
            device_path, measurement, timestamp, val_str
        );
        self.update(&sql).await?;
        Ok(())
    }

    pub async fn batch_insert(&self, records: &[InsertRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        let mut values = Vec::new();
        for record in records {
            let value_str = serde_json::to_string(&record.value)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            values.push(format!("({}, {})", record.timestamp, value_str));
        }

        let sql = format!("INSERT INTO {} (timestamp, value) VALUES {}",
                          records[0].path, values.join(", "));
        self.update(&sql).await?;
        Ok(())
    }

    /// 🚀 工业级关系表单行数据高吞吐写入
    pub async fn insert_table_row(&self, record: &TableRowInsert) -> Result<u64> {
        // 1. 初始化列名容器和值容器，表格模型主时间戳列固定为 `time`
        let mut columns = vec!["time".to_string()];
        let mut values = vec![record.time.to_string()]; // 直接使用 i64 数字作为 TIMESTAMP 写入

        // 2. 组装 Tags（在 SQL 中标签值必须被单引号包裹）
        for (tag_k, tag_v) in &record.tags {
            columns.push(tag_k.clone());
            values.push(format!("'{}'", tag_v.replace("'", "''"))); // 简单防止 SQL 注入
        }

        // 3. 组装 Fields 测点值
        for (field_k, field_v) in &record.fields {
            columns.push(field_k.clone());
            let val_str = match field_v {
                serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")), // 文本类型加单引号
                serde_json::Value::Null => "NULL".to_string(),
                _ => field_v.to_string(), // 数字、布尔直接转字符串
            };
            values.push(val_str);
        }

        // 4. 生成标准关系型 SQL: INSERT INTO table_name (time, tag1, field1) VALUES (1718812800000, 'd1', 23.5)
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            record.table_name,
            columns.join(", "),
            values.join(", ")
        );

        // 5. 投递给带断线自愈能力的 execute 引擎执行
        self.update(&sql).await
    }

    pub async fn delete(&self, path: &str, timestamp: i64) -> Result<()> {
        let sql = format!("DELETE FROM {} WHERE timestamp = {}", path, timestamp);
        self.update(&sql).await?;
        Ok(())
    }

    pub async fn delete_series(&self, path: &str) -> Result<()> {
        let sql = format!("DELETE FROM {}", path);
        self.update(&sql).await?;
        Ok(())
    }

    // ========================================================================
    // 高级查询
    // ========================================================================

    pub async fn query_latest(&self, path: &str, limit: usize) -> Result<QueryResult> {
        let sql = format!("SELECT * FROM {} ORDER BY time DESC LIMIT {}", path, limit);
        self.query(&sql).await
    }

    pub async fn query_range(&self, path: &str, start_time: i64, end_time: i64) -> Result<QueryResult> {
        let sql = format!("SELECT * FROM {} WHERE time >= {} AND time <= {}", path, start_time, end_time);
        self.query(&sql).await
    }

    pub async fn query_aggregate(&self, path: &str, aggregate: &str, start_time: i64, end_time: i64) -> Result<QueryResult> {
        let sql = format!("SELECT {}(*) FROM {} WHERE time >= {} AND time <= {}",
                          aggregate, path, start_time, end_time);
        self.query(&sql).await
    }

    // ========================================================================
    // 元数据管理
    // ========================================================================

    pub async fn show_databases(&self) -> Result<QueryResult> {
        self.query("SHOW DATABASES").await
    }

    pub async fn show_timeseries(&self, path: Option<&str>) -> Result<QueryResult> {
        let sql = match path {
            Some(p) => format!("SHOW TIMESERIES {}", p),
            None => "SHOW TIMESERIES".to_string(),
        };
        self.query(&sql).await
    }

    pub async fn show_storage_groups(&self) -> Result<QueryResult> {
        self.query("SHOW STORAGE GROUP").await
    }

    pub async fn show_devices(&self) -> Result<QueryResult> {
        self.query("SHOW DEVICES").await
    }

    pub async fn show_child_paths(&self, path: &str) -> Result<QueryResult> {
        let sql = format!("SHOW CHILD PATHS {}", path);
        self.query(&sql).await
    }

    pub async fn show_data_nodes(&self) -> Result<QueryResult> {
        self.query("SHOW DATANODES").await
    }

    pub async fn show_config_nodes(&self) -> Result<QueryResult> {
        self.query("SHOW CONFIGNODES").await
    }

    pub async fn show_current_sql_dialect(&self) -> Result<QueryResult> {
        self.query("SHOW CURRENT_SQL_DIALECT").await
    }

    // ========================================================================
    // 系统管理
    // ========================================================================
    async fn show_version(&self) -> Result<QueryResult> {
        self.query("SHOW VERSION").await
    }

    pub async fn version(&self) -> Result<String> {
        let result = self.show_version().await?;
        if !result.rows.is_empty() {
            if let Some(row) = result.rows.first() {
                // 优先获取 "Version" 列
                if let Some(version) = row.fields.get("Version").and_then(|v| v.as_str()) {
                    return Ok(version.to_string());
                }
                // 备选：取第一个值
                if let Some(version) = row.fields.values().next().and_then(|v| v.as_str()) {
                    return Ok(version.to_string());
                }
            }
        }
        Ok("unknown".to_string())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("Client dropped");
    }
}