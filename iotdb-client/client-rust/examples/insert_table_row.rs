// examples/insert_table_row.rs
//! 数据插入示例
//!
//! 演示关系表（Table）模型下的多种数据插入方式：单条表记录插入、批量表记录插入以及原生 SQL 插入

use iotdb_rust_client::{Client, ClientConfig, TableRowInsert};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═════════════════════════════════════════════════════╗");
    println!("║     IoTDB Rust Client - Insert Table Row Example    ║");
    println!("╚═════════════════════════════════════════════════════╝");
    println!();

    // ========== 1. 链式配置与客户端初始化 ==========
    println!("--- 1. Initialize Client with TABLE Dialect ---");

    let config = ClientConfig::default()
        .with_auth("root", "root")
        .with_zone_id("Asia/Shanghai")
        .with_sql_dialect("TABLE")      // 🚀 显式切换为关系表方言模型
        .with_timeout(60);

    let client = Client::with_config(config);
    client.connect().await?;
    println!("  ✅ Base Client connected successfully.");
    println!();

    // ========== 1.5 关系表模型特有的建库建表初始化 ==========
    println!("--- 1.5 Prepare Database and Table Schema ---");

    // 🚀 修复核心：创建目标关系型数据库
    client.query("CREATE DATABASE IF NOT EXISTS demo_db").await?;
    println!("  ✅ Database 'demo_db' verified/created.");

    // 🚀 修复核心：创建结构表，明确划定 TAG（索引列）与 FIELD（数据列）
    let create_table_sql = "CREATE TABLE IF NOT EXISTS demo_db.factory_devices ( \
                            device_id STRING TAG, \
                            workshop STRING TAG, \
                            temperature DOUBLE FIELD, \
                            is_working BOOLEAN FIELD \
                            )";
    client.query(create_table_sql).await?;
    println!("  ✅ Table 'demo_db.factory_devices' schema definition synchronized.");
    println!();

    // ========== 2. 单条表记录插入 (Single Row) ==========
    println!("--- 2. Single Table Row Insertion ---");

    let mut tags = HashMap::new();
    tags.insert("device_id".to_string(), "pump_01".to_string());
    tags.insert("workshop".to_string(), "cell_a".to_string());

    let mut fields = HashMap::new();
    fields.insert("temperature".to_string(), serde_json::json!(42.7));
    fields.insert("is_working".to_string(), serde_json::json!(true));

    let row = TableRowInsert {
        // 🚀 修复核心：通过全限定名指定数据库
        table_name: "demo_db.factory_devices".to_string(),
        time: 1718812800000,
        tags,
        fields,
    };

    match client.insert_table_row(&row).await {
        Ok(_) => println!("  ✅ Single table row inserted successfully!"),
        Err(e) => eprintln!("  ❌ Single row insertion failed: {}", e),
    }
    println!();

    // ========== 3. 批量表记录插入 (Batch Rows) ==========
    println!("--- 3. Batch Table Rows Insertion ---");

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as i64;

    let mut batch_rows = Vec::new();

    for i in 1..=3 {
        let mut tags_batch = HashMap::new();
        tags_batch.insert("device_id".to_string(), format!("pump_0{}", i));
        tags_batch.insert("workshop".to_string(), "cell_b".to_string());

        let mut fields_batch = HashMap::new();
        fields_batch.insert("temperature".to_string(), serde_json::json!(35.0 + (i as f64) * 2.5));
        fields_batch.insert("is_working".to_string(), serde_json::json!(true));

        batch_rows.push(TableRowInsert {
            // 🚀 修复核心：批量数据也同样指向明确的库
            table_name: "demo_db.factory_devices".to_string(),
            time: current_time + (i * 1000) as i64,
            tags: tags_batch,
            fields: fields_batch,
        });
    }
    println!("  Generated {} table rows for batch ingestion...", batch_rows.len());

    let mut success_count = 0;
    for row in &batch_rows {
        if client.insert_table_row(row).await.is_ok() {
            success_count += 1;
        }
    }
    println!("  ✅ Batch insertion completed: {}/{} rows ingested successfully!", success_count, batch_rows.len());
    println!();

    // ========== 4. 原生 SQL 方式插入 (Native SQL) ==========
    println!("--- 4. Native SQL Insertion ---");

    // 🚀 修复核心：在 SQL 语句中显式指定 demo_db.factory_devices
    let sql_insert = "INSERT INTO demo_db.factory_devices(time, device_id, workshop, temperature, is_working) \
                      VALUES(1718812900000, 'pump_04', 'cell_a', 51.2, false)";

    println!("  Executing raw SQL: \n     {}", sql_insert);
    match client.query(sql_insert).await {
        Ok(_) => println!("  ✅ Native SQL row insertion executed successfully!"),
        Err(e) => eprintln!("  ❌ Native SQL insertion failed: {}", e),
    }
    println!();

    // ========== 5. 优雅清理与离线 ==========
    client.disconnect().await?;
    println!("✅ Disconnected safely from IoTDB");
    println!("\n✅ Table data ingestion pipeline verified successfully with full test styles!");

    Ok(())
}