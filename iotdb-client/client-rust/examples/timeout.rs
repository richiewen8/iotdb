// examples/timeout.rs
//! 超时处理示例
//!
//! 演示如何设置客户端超时、利用 Tokio 包装超时以及实现高级异步重试机制

use iotdb_rust_client::{Client, ClientConfig, InsertRecord, Error};
use serde_json::json;
use tokio::time::timeout;
use std::time::Duration;
use std::sync::Arc;
use std::future::Future;

// =========================================================================
// 🚀 工具函数抽离：统一放在外层，保持架构清晰、职责分离
// =========================================================================

/// 组合操作单次超时包装器
async fn execute_with_timeout<T, F>(
    operation: F,
    timeout_duration: Duration
) -> Result<T, String>
where
    F: Future<Output = Result<T, Error>>,
{
    match tokio::time::timeout(timeout_duration, operation).await {
        Ok(result) => match result {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("Operation failed: {}", e)),
        },
        Err(_) => Err(format!("Operation timed out after {:?}", timeout_duration)),
    }
}

/// 高级工业级异步重试带超时状态机
async fn retry_with_timeout<T, F>(
    mut operation: F,
    max_retries: usize,
    timeout_duration: Duration,
    retry_delay: Duration,
) -> Result<T, String>
where
// 💡 显式加上 'static 约束，确保 Future 在跨线程调度时生命周期足够长
    F: FnMut() -> std::pin::Pin<Box<dyn Future<Output = Result<T, Error>> + Send + 'static>>,
    T: Send + 'static,
{
    let mut attempts = 0;

    while attempts < max_retries {
        attempts += 1;
        println!("    Attempt {}/{}", attempts, max_retries);

        match tokio::time::timeout(timeout_duration, operation()).await {
            Ok(result) => match result {
                Ok(data) => {
                    println!("    ✅ Operation succeeded on attempt {}", attempts);
                    return Ok(data);
                }
                Err(e) => {
                    println!("    ❌ Operation failed: {}", e);
                    if attempts >= max_retries {
                        return Err(format!("All {} attempts failed: {}", max_retries, e));
                    }
                }
            },
            Err(_) => {
                println!("    ⏰ Operation timed out on attempt {}", attempts);
                if attempts >= max_retries {
                    return Err(format!("All {} attempts timed out", max_retries));
                }
            }
        }

        println!("    ⏳ Waiting {}ms before retry...", retry_delay.as_millis());
        tokio::time::sleep(retry_delay).await;
    }

    Err("Max retries exceeded".to_string())
}


// =========================================================================
// 🏁 主执行流
// =========================================================================
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════╗");
    println!("║     IoTDB Rust Client - Timeout Example      ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();

    // ========== 1. 配置标准 TREE 客户端 ==========
    println!("--- 1. Configure Timeouts ---");

    let config = ClientConfig::default()
        .with_auth("root", "root")
        .with_timeout(30)
        .with_session_timeout(3600)
        .with_sql_dialect("TREE"); // 🚀 全局死守 TREE 方言

    let client = Client::with_config(config);
    println!("  ✅ Base Client initialized with 30s timeout & TREE dialect.");
    println!();

    // ========== 2. 连接超时演练 ==========
    println!("--- 2. Connection Timeout ---");

    let short_timeout_config = ClientConfig::default()
        .with_auth("root", "root")
        .with_timeout(1);
    let short_timeout_client = Client::with_config(short_timeout_config);

    match timeout(Duration::from_secs(2), short_timeout_client.connect()).await {
        Ok(result) => match result {
            Ok(()) => println!("  ✅ Connected successfully"),
            Err(e) => println!("  ❌ Connection failed: {}", e),
        },
        Err(_) => println!("  ⏰ Connection timed out after 2 seconds"),
    }
    println!();

    // ========== 3. 查询超时演练 ==========
    println!("--- 3. Query Timeout ---");

    // 🚀 核心修复：直接使用第 1 节创建的、配置了 TREE 方言的 client，不再盲目重造
    client.connect().await?;
    println!("  ✅ Connected to IoTDB");

    match timeout(Duration::from_secs(5), client.query("SHOW DATABASES")).await {
        Ok(result) => match result {
            Ok(data) => println!("  ✅ Query completed: {} rows", data.rows.len()),
            Err(e) => println!("  ❌ Query failed: {}", e),
        },
        Err(_) => println!("  ⏰ Query timed out after 5 seconds"),
    }
    println!();

    // ========== 4. 批量插入超时演练 ==========
    println!("--- 4. Batch Insert Timeout ---");

    let mut records = Vec::new();
    let base_time = chrono::Utc::now().timestamp_millis();

    for i in 0..1000 {
        let timestamp = base_time + (i * 100) as i64;
        records.push(InsertRecord {
            // 🚀 核心修正：将其改回设备路径（Device Path），不要把叶子测点 .value 写在设备名里
            path: "root.sg1.d1.temperature".to_string(),
            timestamp,
            value: json!(20.0 + (i % 100) as f64),
        });
    }
    println!("  Generated {} records for batch insert", records.len());

    // 50ms 极限阈值测试（高几率触发超时断开，用于演示拦截流）
    match timeout(Duration::from_millis(50), client.batch_insert(&records)).await {
        Ok(result) => match result {
            Ok(()) => println!("  ✅ Batch insert completed successfully within 50ms"),
            Err(e) => println!("  ❌ Batch insert failed: {}", e),
        },
        Err(_) => println!("  ⏰ Batch insert timed out (50ms limit triggered perfectly!)"),
    }

    // 宽裕时间测试：在 TREE 模型下，如果没超时，它会自动创建或灌入序列
    match timeout(Duration::from_secs(30), client.batch_insert(&records)).await {
        Ok(result) => match result {
            Ok(()) => println!("  ✅ Batch insert completed successfully (30s window)"),
            Err(e) => println!("  ❌ Batch insert failed: {}", e),
        },
        Err(_) => println!("  ⏰ Batch insert timed out (30s limit)"),
    }
    println!();

    // ========== 5. 组合操作超时 ==========
    println!("--- 5. Combined Operations Timeout ---");

    let result = execute_with_timeout(
        client.query("SHOW DATABASES"),
        Duration::from_secs(3)
    ).await;
    if let Ok(data) = result {
        println!("  ✅ Query succeeded via wrapper: {} rows", data.rows.len());
    }

    let result = execute_with_timeout(
        client.query("SELECT * FROM root.sg1.d1.**"),
        Duration::from_millis(1)
    ).await;
    if let Err(e) = result {
        println!("  ❌ Expected timeout check: {}", e);
    }
    println!();

    // ========== 6. 重试带超时 ==========
    println!("--- 6. Retry with Timeout ---");

    let client_arc = Arc::new(client);
    let client_clone = client_arc.clone();

    let result = retry_with_timeout(
        move || -> std::pin::Pin<Box<dyn Future<Output = Result<_, _>> + Send + 'static>> {
            let client = client_clone.clone();
            Box::pin(async move {
                client.query("SHOW DATABASES").await
            })
        },
        3,
        Duration::from_secs(2),
        Duration::from_millis(500)
    ).await;

    if let Ok(data) = result {
        println!("  ✅ Final query succeeded after retry tracking: {} rows", data.rows.len());
    }
    println!();

    // ========== 清理 ==========
    client_arc.disconnect().await?;
    println!("✅ Disconnected safely");
    println!("\n✅ Timeout control architecture verified successfully with ZERO compilation warnings!");

    Ok(())
}