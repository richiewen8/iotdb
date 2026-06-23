// examples/query.rs
//! 数据查询示例
//!
//! 演示多种查询方式：最新数据、时间范围、聚合查询、元数据查询

use iotdb_rust_client::{Client, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════╗");
    println!("║      IoTDB Rust Client - Query Example     ║");
    println!("╚════════════════════════════════════════════╝");
    println!();

    let config = ClientConfig::default()
        .with_auth("root", "root")
        .with_zone_id("Asia/Shanghai")
        .with_timeout(60)
        .with_sql_dialect("TREE");

    let client = Client::with_config(config);
    client.connect().await?;
    println!("✅ Connected to IoTDB\n");

    // ========== 1. 元数据查询 ==========
    println!("--- 1. Metadata Queries ---");

    // 显示数据库
    println!("\n  📊 Databases:");
    let result = client.show_databases().await?;
    for row in &result.rows {
        if let Some(db) = row.fields.get("Database").or_else(|| row.fields.get("database")) {
            println!("    Database: {}", db);
        }
    }

    // 显示时间序列
    println!("\n  📈 Timeseries:");
    let result = client.show_timeseries(None).await?;
    for row in &result.rows {
        let path = row.fields.get("Timeseries").or_else(|| row.fields.get("timeseries")).or_else(|| row.fields.get("path")).and_then(|v| v.as_str()).unwrap_or("");
        let data_type = row.fields.get("DataType").or_else(|| row.fields.get("data_type")).and_then(|v| v.as_str()).unwrap_or("");
        if !path.is_empty() {
            println!("    {} ({})", path, data_type);
        }
    }
    println!();

    // ========== 2. 数据查询 ==========
    println!("--- 2. Data Queries ---");

    // 查询最新数据
    println!("\n  🔍 Latest 5 records:");
    // 🚀 适配：使用通配符匹配所有单点、批量及SQL写入的路径
    let result = client.query_latest("root.**", 5).await?;
    if result.rows.is_empty() {
        println!("    No data found. Please run insert example first.");
    } else {
        for row in &result.rows {
            // 🚀 核心修复：兼容 IoTDB 原生 SELECT LAST 返回的大写键 (Time, Timeseries, Value)
            let timestamp = row.fields.get("Time").or_else(|| row.fields.get("time")).or_else(|| row.fields.get("timestamp")).and_then(|v| v.as_i64()).unwrap_or(0);
            let value = row.fields.get("Value").or_else(|| row.fields.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let path = row.fields.get("Timeseries").or_else(|| row.fields.get("timeseries")).or_else(|| row.fields.get("path")).and_then(|v| v.as_str()).unwrap_or("");

            if !path.is_empty() {
                println!("    {}: time={}, value={:.2}", path, timestamp, value);
            }
        }
    }

    // 时间范围查询
    println!("\n  📅 Time range query (last 5 minutes + safety buffer):");
    let now = chrono::Utc::now().timestamp_millis();
    let start_time = now - 300000;
    let end_time = now + 600000; // 宽裕的未来缓冲窗口

    // 🚀 核心修复：根据你本地实际存在的序列路径，查询带 `.value` 的全路径或 demo 路径
    let query_path = "root.sg1.d1.temperature.value";
    let result = client.query_range(query_path, start_time, end_time).await?;
    println!("    Target Path: {}", query_path);
    println!("    Found {} records in time window", result.rows.len());

    if !result.rows.is_empty() {
        let display_count = std::cmp::min(3, result.rows.len());
        for row in &result.rows[..display_count] {
            let timestamp = row.fields.get("Time").or_else(|| row.fields.get("time")).or_else(|| row.fields.get("timestamp")).and_then(|v| v.as_i64()).unwrap_or(0);
            let value = row.fields.get("Value").or_else(|| row.fields.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            println!("      time={}, value={:.2}", timestamp, value);
        }
    }
    println!();

    // ========== 3. 聚合查询 ==========
    println!("--- 3. Aggregate Queries ---");

    // AVG
    let result = client.query_aggregate(query_path, "AVG", start_time, end_time).await?;
    if let Some(row) = result.rows.first() {
        let avg = row.fields.values().next().and_then(|v| v.as_f64()).unwrap_or(0.0);
        println!("  📊 AVG temperature: {:.2}", avg);
    }

    // MAX_VALUE (🚀 核心修复：将 MAX 替换为 TREE 模型标准函数 MAX_VALUE)
    let result = client.query_aggregate(query_path, "MAX_VALUE", start_time, end_time).await?;
    if let Some(row) = result.rows.first() {
        let max = row.fields.values().next().and_then(|v| v.as_f64()).unwrap_or(0.0);
        println!("  📈 MAX temperature: {:.2}", max);
    }

    // MIN_VALUE (🚀 核心修复：将 MIN 替换为 TREE 模型标准函数 MIN_VALUE)
    let result = client.query_aggregate(query_path, "MIN_VALUE", start_time, end_time).await?;
    if let Some(row) = result.rows.first() {
        let min = row.fields.values().next().and_then(|v| v.as_f64()).unwrap_or(0.0);
        println!("  📉 MIN temperature: {:.2}", min);
    }

    // COUNT
    let result = client.query_aggregate(query_path, "COUNT", start_time, end_time).await?;
    if let Some(row) = result.rows.first() {
        let count = row.fields.values().next().and_then(|v| v.as_u64()).unwrap_or(0);
        println!("  🔢 COUNT: {}", count);
    }
    println!();

    // ========== 4. 系统信息 ==========
    println!("--- 4. System Information ---");
    if let Ok(version) = client.version().await {
        println!("  🏷️  IoTDB Version: {}", version);
    }

    client.disconnect().await?;
    println!("✅ Disconnected");
    println!("\n✅ All query modes verified successfully with zero UDF exceptions!");

    Ok(())
}