// examples/error_handling.rs
//! 错误处理示例
//!
//! 演示如何处理各种错误情况：连接错误、查询错误、超时错误等

use iotdb_rust_client::{Client, ClientConfig, Error, QueryResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════╗");
    println!("║  IoTDB Rust Client - Error Handling Example  ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();

    // 创建带认证的标准配置，确保后续功能测试不会被中途拦截
    let standard_config = ClientConfig::default()
        .with_auth("root", "root")
        .with_sql_dialect("TREE");

    // ========== 1. 连接错误 ==========
    println!("--- 1. Connection Errors ---");

    // 错误的地址
    println!("  Trying to connect to invalid address...");
    let invalid_host_config = ClientConfig::default().with_server_addr("invalid.host", 6667);
    let client_invalid_host = Client::with_config(invalid_host_config);
    match client_invalid_host.connect().await {
        Ok(()) => println!("  ✅ Connected (unexpected)"),
        Err(e) => println!("  ❌ Connection failed as expected: {}", e),
    }
    println!();

    // 错误的端口
    println!("  Trying to connect to wrong port...");
    let wrong_port_config = ClientConfig::default().with_server_addr("localhost", 9999);
    let client_wrong_port = Client::with_config(wrong_port_config);
    match client_wrong_port.connect().await {
        Ok(()) => println!("  ✅ Connected (unexpected)"),
        Err(e) => println!("  ❌ Connection failed as expected: {}", e),
    }
    println!();

    // ========== 2. 查询错误（未连接） ==========
    println!("--- 2. Query Errors (Not Connected) ---");
    let client_not_connected = Client::with_config(standard_config.clone());

    match client_not_connected.query("SHOW DATABASES").await {
        Ok(_) => println!("  ✅ Query succeeded! (Verified: SDK supports lazy auto-connection)"),
        Err(e) => println!("  ❌ Query failed: {}", e),
    }
    println!();

    // ========== 3. 查询错误（无效SQL） ==========
    println!("--- 3. Query Errors (Invalid SQL) ---");
    let client = Client::with_config(standard_config.clone());
    match client.connect().await {
        Ok(()) => println!("  ✅ Connected to IoTDB"),
        Err(e) => {
            println!("  ❌ Connection failed: {}", e);
            return Ok(());
        }
    }

    match client.query("INVALID SQL STATEMENT").await {
        Ok(_) => println!("  ✅ Query succeeded (unexpected)"),
        Err(e) => match e {
            Error::Execution(msg) => println!("  ❌ Execution error as expected:\n     {}", msg.trim()),
            _ => println!("  ❌ Unexpected error type: {}", e),
        }
    }
    println!();

    // ========== 4. 认证错误仿真 ==========
    println!("--- 4. Authentication Errors ---");

    let wrong_auth_config = ClientConfig::default()
        .with_auth("root", "WRONG_PASSWORD")
        .with_sql_dialect("TREE");
    let client_bad_auth = Client::with_config(wrong_auth_config);

    match client_bad_auth.connect().await {
        Ok(()) => {
            println!("  ✅ Connected (Warning: Server might not have authentication enabled)");
        }
        Err(e) => match e {
            Error::Auth(msg) => println!("  ❌ Authentication error caught: {}", msg),
            // 🚀 核心对齐：SDK 真实返回的是 Execution 错误
            Error::Execution(msg) if msg.contains("拒绝登录") => {
                println!("  ❌ Authentication rejected by server as expected:\n     {}", msg.trim());
            }
            Error::Connection(msg) => println!("  ❌ Connection refused: {}", msg),
            _ => println!("  ❌ Other error: {}", e),
        }
    }
    println!();

    // ========== 5. 错误恢复与断线重连 ==========
    println!("--- 5. Error Recovery ---");
    println!("  Demonstrating error recovery with retry logic...");

    let max_retries = 3;
    let mut attempt = 0;

    loop {
        attempt += 1;
        println!("  Attempt {}/{}", attempt, max_retries);

        match client.query("SHOW DATABASES").await {
            Ok(result) => {
                println!("  ✅ Query succeeded on attempt {}", attempt);
                println!("  Found {} databases", result.rows.len());
                break;
            }
            Err(e) => {
                println!("  ❌ Query failed: {}", e);

                if attempt >= max_retries {
                    println!("  ❌ Max retries exceeded, giving up");
                    break;
                }

                if let Error::Connection(_) = e {
                    println!("  🔄 Connection lost, attempting reconnection...");
                    match client.connect().await {
                        Ok(()) => println!("  ✅ Reconnected successfully"),
                        Err(err) => println!("  ❌ Failed to reconnect: {}", err),
                    }
                }

                println!("  ⏳ Waiting 500ms before next retry...");
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
    println!();

    // ========== 6. 优雅错误处理模式 ==========
    println!("--- 6. Elegant Error Handling Patterns ---");

    // 模式1: 使用 ? 操作符
    println!("  Pattern 1: Using ? operator");
    match do_something(&client).await {
        Ok(msg) => println!("  ✅ Success: {}", msg),
        Err(e) => println!("  ❌ Error forwarded: {}", e),
    }
    println!();

    // 模式2: 使用 unwrap_or_else
    println!("  Pattern 2: Using unwrap_or_else");
    let version = client.version().await.unwrap_or_else(|e| {
        println!("    ⚠️  Failed to get version: {}", e);
        "unknown_version_fallback".to_string()
    });
    println!("  Version: {}", version);
    println!();

    // 模式3: 使用 map_err
    println!("  Pattern 3: Using map_err");
    let result = client.query("SHOW DATABASES").await
        .map_err(|e| format!("Wrapped System Error Context -> {}", e))
        .unwrap_or_else(|e| {
            println!("    ⚠️  Intercepted: {}", e);
            // 🚀 显式手动初始化，避开对 Default 的依赖
            QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: None,
            }
        });
    println!("  Result rows: {}", result.rows.len());
    println!();

    let _ = client.disconnect().await;
    println!("✅ Disconnected safely");
    println!("\n✅ All error capture pipelines executed cleanly!");

    Ok(())
}

async fn do_something(client: &Client) -> Result<String, Box<dyn std::error::Error>> {
    let result = client.query("SHOW DATABASES").await?;
    Ok(format!("Found {} databases", result.rows.len()))
}