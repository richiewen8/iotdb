//! 元数据操作示例
//!
//! 演示如何管理 IoTDB 的元数据：创建、查询、删除等

use iotdb_rust_client::{Client, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════╗");
    println!("║   IoTDB Rust Client - Metadata Example       ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();

    let config = ClientConfig::default()
        .with_auth("root", "root")
        .with_zone_id("Asia/Shanghai")
        .with_timeout(60)
        .with_sql_dialect("TREE");

    let client = Client::with_config(config);
    client.connect().await?;
    println!("✅ Connected to IoTDB\n");

    // ========== 1. 创建存储组 ==========
    println!("--- 1. Create Storage Group ---");
    let sql = "CREATE STORAGE GROUP root.sg_test";
    match client.execute(sql).await {
        Ok(_) => println!("  ✅ Created: {}", sql),
        Err(e) => println!("  ⚠️  Create storage group: {}", e),
    }
    println!();

    // ========== 2. 创建时间序列 ==========
    println!("--- 2. Create Timeseries ---");
    let ts_defs = vec![
        "CREATE TIMESERIES root.sg_test.d1.temperature WITH DATATYPE=FLOAT, ENCODING=GORILLA",
        "CREATE TIMESERIES root.sg_test.d1.humidity WITH DATATYPE=FLOAT, ENCODING=GORILLA",
        "CREATE TIMESERIES root.sg_test.d1.pressure WITH DATATYPE=FLOAT, ENCODING=GORILLA",
        "CREATE TIMESERIES root.sg_test.d2.temperature WITH DATATYPE=DOUBLE, ENCODING=GORILLA",
        "CREATE TIMESERIES root.sg_test.d2.voltage WITH DATATYPE=INT32, ENCODING=GORILLA",
    ];

    for ts_def in ts_defs {
        match client.execute(ts_def).await {
            Ok(_) => println!("  ✅ Created: {}", ts_def),
            Err(e) => println!("  ⚠️  Create timeseries: {}", e),
        }
    }
    println!();

    // ========== 3. 显示存储组 ==========
    println!("--- 3. Show Storage Groups ---");
    let result = client.show_storage_groups().await?;
    if result.rows.is_empty() {
        println!("  No storage groups found");
    } else {
        for row in &result.rows {
            for (key, value) in &row.fields {
                println!("  {}: {}", key, value);
            }
        }
    }
    println!();

    // ========== 4. 显示时间序列 ==========
    println!("--- 4. Show Timeseries ---");
    let result = client.show_timeseries(Some("root.sg_test.*")).await?;
    if result.rows.is_empty() {
        println!("  No timeseries found in root.sg_test.*");
    } else {
        for row in &result.rows {
            let path = row.fields.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let data_type = row.fields.get("data_type").and_then(|v| v.as_str()).unwrap_or("");
            let encoding = row.fields.get("encoding").and_then(|v| v.as_str()).unwrap_or("");
            println!("  {}: type={}, encoding={}", path, data_type, encoding);
        }
    }
    println!();

    // ========== 5. 显示设备 ==========
    println!("--- 5. Show Devices ---");
    let result = client.show_devices().await?;
    if result.rows.is_empty() {
        println!("  No devices found");
    } else {
        for row in &result.rows {
            for (key, value) in &row.fields {
                if key == "device" {
                    println!("  Device: {}", value);
                }
            }
        }
    }
    println!();

    // ========== 6. 显示子路径 ==========
    println!("--- 6. Show Child Paths ---");
    let result = client.show_child_paths("root.sg_test").await?;
    if result.rows.is_empty() {
        println!("  No child paths found");
    } else {
        for row in &result.rows {
            for (key, value) in &row.fields {
                println!("  {}: {}", key, value);
            }
        }
    }
    println!();

    // ========== 7. 插入测试数据 ==========
    println!("--- 7. Insert Test Data ---");
    let timestamp = chrono::Utc::now().timestamp_millis();

    let _ = client.insert("root.sg_test.d1", "temperature",timestamp, serde_json::json!(36.5)).await;
    let _ = client.insert("root.sg_test.d1", "humidity", timestamp, serde_json::json!(65.0)).await;
    let _ = client.insert("root.sg_test.d1", "pressure",timestamp, serde_json::json!(1013.25)).await;

    println!("  ✅ Inserted test data at {}", timestamp);
    println!();

    // ========== 8. 查询数据验证 ==========
    println!("--- 8. Verify Data ---");
    let result = client.query_latest("root.sg_test.d1.*", 5).await?;
    if result.rows.is_empty() {
        println!("  No data found");
    } else {
        println!("  Found {} records:", result.rows.len());
        for row in &result.rows {
            let timestamp = row.fields.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
            let value = row.fields.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let path = row.fields.get("path").and_then(|v| v.as_str()).unwrap_or("");
            println!("    {}: time={}, value={:.2}", path, timestamp, value);
        }
    }
    println!();

    // ========== 9. 删除时间序列 ==========
    println!("--- 9. Delete Timeseries ---");
    match client.delete_series("root.sg_test.d2.voltage").await {
        Ok(_) => println!("  ✅ Deleted: root.sg_test.d2.voltage"),
        Err(e) => println!("  ⚠️  Delete timeseries: {}", e),
    }
    println!();

    // ========== 10. 确认删除 ==========
    println!("--- 10. Verify Deletion ---");
    let result = client.show_timeseries(Some("root.sg_test.d2.*")).await?;
    if result.rows.is_empty() {
        println!("  ✅ No timeseries found in root.sg_test.d2 (deleted successfully)");
    } else {
        println!("  Found {} timeseries", result.rows.len());
        for row in &result.rows {
            let path = row.fields.get("path").and_then(|v| v.as_str()).unwrap_or("");
            println!("    {}", path);
        }
    }
    println!();

    // ========== 11. 清理测试数据 ==========
    println!("--- 11. Cleanup ---");
    let test_ts = vec![
        "root.sg_test.d1.temperature",
        "root.sg_test.d1.humidity",
        "root.sg_test.d1.pressure",
        "root.sg_test.d2.temperature",
    ];

    for ts in test_ts {
        let _ = client.delete_series(ts).await;
        println!("  ✅ Deleted: {}", ts);
    }

    let _ = client.execute("DELETE STORAGE GROUP root.sg_test").await;
    println!("  ✅ Deleted storage group: root.sg_test");
    println!();

    client.disconnect().await?;
    println!("✅ Disconnected");
    println!("\n✅ Metadata example completed successfully!");

    Ok(())
}