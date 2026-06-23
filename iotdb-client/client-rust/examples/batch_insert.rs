//! 批量插入示例
//!
//! 演示大规模数据批量插入和性能测试

use iotdb_rust_client::{Client, ClientConfig, InsertRecord};
use rand::RngExt;
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════╗");
    println!("║   IoTDB Rust Client - Batch Insert Example   ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();

    // 🚀 强烈建议：测试前先去 IoTDB CLI 执行 `DELETE DATABASE root.sg1;` 清空旧数据
    let config = ClientConfig::default()
        .with_auth("root", "root")
        .with_zone_id("Asia/Shanghai")
        .with_timeout(60)
        .with_sql_dialect("TREE");

    let client = Client::with_config(config);
    client.connect().await?;
    println!("✅ Connected to IoTDB\n");

    let mut rng = rand::rng();

    // ========== 配置 ==========
    println!("--- Configuration ---");
    let total_records = 1000;
    let batch_size = 100;
    let paths = vec![
        "root.sg1.d1.temperature",
        "root.sg1.d1.humidity",
        "root.sg1.d1.pressure",
        "root.sg1.d2.temperature",
        "root.sg1.d2.humidity",
    ];
    println!("  Total records: {}", total_records);
    println!("  Batch size: {}", batch_size);
    println!("  Paths: {:?}", paths);
    println!();

    // 💡 提示：这里去掉了原有的 CREATE TIMESERIES 块，利用 IoTDB 的自动智能创建机制

    // ========== 生成数据 ==========
    println!("--- Generating Data ---");
    let start_gen = Instant::now();
    let mut all_records = Vec::with_capacity(total_records);
    let base_time = chrono::Utc::now().timestamp_millis();

    for i in 0..total_records {
        // 时间戳向后递增 100ms
        let timestamp = base_time + (i * 100) as i64;
        let path_idx = i % paths.len();
        let path = paths[path_idx];

        let value = match path_idx {
            0 => json!(20.0 + rng.random_range(-5.0..10.0)),
            1 => json!(50.0 + rng.random_range(-20.0..20.0)),
            2 => json!(1013.0 + rng.random_range(-10.0..10.0)),
            3 => json!(25.0 + rng.random_range(-5.0..10.0)),
            _ => json!(60.0 + rng.random_range(-20.0..20.0)),
        };

        all_records.push(InsertRecord {
            path: path.to_string(),
            timestamp,
            value,
        });
    }

    let gen_duration = start_gen.elapsed();
    println!("  ✅ Generated {} records in {:?}", all_records.len(), gen_duration);
    println!("  Rate: {:.0} records/sec",
             all_records.len() as f64 / gen_duration.as_secs_f64());
    println!();

    // ========== 批量插入 ==========
    println!("--- Batch Insert ---");
    let start_insert = Instant::now();
    let total_batches = (all_records.len() + batch_size - 1) / batch_size;
    let mut inserted = 0;

    for (batch_num, chunk) in all_records.chunks(batch_size).enumerate() {
        let batch_start = Instant::now();

        // 执行客户端的批量 RPC 投递
        client.batch_insert(chunk).await?;

        let batch_duration = batch_start.elapsed();
        inserted += chunk.len();

        println!("  Batch {}/{}: {} records in {:?} (avg {:?} per record)",
                 batch_num + 1, total_batches, chunk.len(),
                 batch_duration, batch_duration / chunk.len() as u32);
    }

    let insert_duration = start_insert.elapsed();
    println!();
    println!("  ✅ Inserted {} records in {:?}", inserted, insert_duration);
    println!("  Average Throughput: {:.0} records/sec",
             inserted as f64 / insert_duration.as_secs_f64());
    println!();

    // ========== 验证最新数据 ==========
    println!("--- Verification ---");
    let result = client.query_latest("root.sg1.d1.*", 10).await?;

    if result.rows.is_empty() {
        println!("  ⚠️ 没有找到记录。如果刚清空过数据库，请确保 batch_insert 底层未报错。");
    } else {
        println!("  ✅ Retrieved {} latest records", result.rows.len());
        let display_count = std::cmp::min(5, result.rows.len());
        for row in &result.rows[..display_count] {
            println!("    Row Data: {:?}", row.fields);
        }
        if result.rows.len() > 5 {
            println!("    ... and {} more", result.rows.len() - 5);
        }
    }
    println!();

    // ========== 统计（覆盖整个未来的生成时间段） ==========
    println!("--- Statistics ---");
    // 🚀 核心修复：结束时间必须包含整个模拟序列的最大时间
    let end_time = base_time + (total_records * 100) as i64 + 1000;

    let stats = client.query_aggregate("root.sg1.d1.*", "COUNT", base_time, end_time).await?;
    if stats.rows.is_empty() {
        println!("  No statistics available (Check time range)");
    } else {
        for row in &stats.rows {
            for (key, value) in &row.fields {
                println!("  {}: {}", key, value);
            }
        }
    }
    println!();

    client.disconnect().await?;
    println!("✅ Disconnected");
    println!("\n✅ Batch insert example completed successfully!");

    Ok(())
}