//! 数据插入示例
//!
//! 演示多种数据插入方式：单条插入、批量插入、原生SQL插入

use iotdb_rust_client::{Client, ClientConfig, InsertRecord};
use rand::RngExt;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════╗");
    println!("║     IoTDB Rust Client - Insert Example    ║");
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

    let mut rng = rand::rng();

    // ========== 方式1：单条插入（面向多测点设备模型） ==========
    println!("--- 1. Single Insert ---");
    let timestamp = chrono::Utc::now().timestamp_millis();
    let single_device = "root.demo.single"; // 🚀 独立路由设备

    client.insert(single_device, "temperature", timestamp, json!(36.5)).await?;
    println!("  ✅ Inserted temperature: 36.5 at {}", timestamp);

    client.insert(single_device, "humidity", timestamp + 100, json!(65.0)).await?;
    println!("  ✅ Inserted humidity: 65.0 at {}", timestamp + 100);

    client.insert(single_device, "pressure", timestamp + 200, json!(1013.25)).await?;
    println!("  ✅ Inserted pressure: 1013.25 at {}", timestamp + 200);
    println!();


    // ========== 方式2：批量插入（面向传统全路径测点模型） ==========
    println!("--- 2. Batch Insert ---");
    let base_time = chrono::Utc::now().timestamp_millis();
    let mut records = Vec::new();

    for i in 0..5 {
        let time = base_time + (i * 1000) as i64;
        let temperature = 20.0 + rng.random_range(-5.0..10.0);
        let humidity = 50.0 + rng.random_range(-20.0..20.0);

        // 🚀 路由到 batch 虚拟设备，避免与 single 冲突
        records.push(InsertRecord {
            path: "root.demo.batch.temperature".to_string(),
            timestamp: time,
            value: json!(temperature),
        });
        records.push(InsertRecord {
            path: "root.demo.batch.humidity".to_string(),
            timestamp: time,
            value: json!(humidity),
        });
    }

    println!("  Generated {} records", records.len());
    client.batch_insert(&records).await?;
    println!("  ✅ Batch inserted {} records", records.len());
    println!();


    // ========== 方式3：原生SQL插入（ALIGNED 多列对齐高吞吐模式） ==========
    println!("--- 3. Native SQL Insert ---");
    let time = chrono::Utc::now().timestamp_millis();
    let sql_device = "root.demo.sql"; // 🚀 独立路由设备

    // 🔥 工业修复：时间列关键字必须为 `time`，而不是 `timestamp`
    // 🔥 优化：将多列对齐（ALIGNED）合并为单条标准语句，极大减少 RPC 交互
    let sql_aligned = format!(
        "INSERT INTO {}(time, temperature, humidity, pressure) ALIGNED VALUES ({}, {}, {}, {})",
        sql_device, time, 25.5, 55.0, 1010.5
    );
    client.execute(&sql_aligned).await?;

    println!("  ✅ Executed native SQL ALIGNED insert for {}", sql_device);
    println!();


    // ========== 验证插入结果 ==========
    println!("--- Verification ---");

    // 1. 验证方式 1 的数据
    println!("  [Checking Single Insert Mode]");
    let res_single = client.query_latest("root.demo.single.*", 3).await?;
    for row in res_single.rows {
        println!("    Row: {:?}", row.fields);
    }

    // 2. 验证方式 2 的数据
    println!("  [Checking Batch Insert Mode]");
    let res_batch = client.query_latest("root.demo.batch.*", 3).await?;
    for row in res_batch.rows {
        println!("    Row: {:?}", row.fields);
    }

    // 3. 验证方式 3 的数据
    println!("  [Checking Native SQL Mode]");
    let res_sql = client.query_latest("root.demo.sql.*", 3).await?;
    for row in res_sql.rows {
        println!("    Row: {:?}", row.fields);
    }
    println!();

    client.disconnect().await?;
    println!("✅ Disconnected");
    println!("\n✅ All insert styles verified successfully with zero schema conflicts!");

    Ok(())
}