// examples/connect.rs
//! 连接和断开测试示例
//!
//! 演示如何创建客户端、连接到 IoTDB、检查连接状态并断开连接

use iotdb_rust_client::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════╗");
    println!("║     IoTDB Rust Client - Connection Test    ║");
    println!("╚════════════════════════════════════════════╝");
    println!();

    // ========== 创建客户端 ==========
    let client = Client::new("localhost", 6667);
    println!("✅ Client created: {}:{}", client.config.host, client.config.port);
    println!();

    // ========== 测试连接前状态 ==========
    println!("--- Before Connection ---");
    println!("Connected: {}", client.is_connected().await);
    println!();

    // ========== 连接 ==========
    println!("--- Connecting ---");
    match client.connect().await {
        Ok(()) => {
            println!("✅ Connected to IoTDB at {}:{}",
                     client.config.host, client.config.port);
            println!("Connected: {}", client.is_connected().await);
        }
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            return Ok(());
        }
    }
    println!();

    // ========== 获取连接信息 ==========
    println!("--- Connection Info ---");
    println!("Session ID: {:?}", client.get_session_id().await);
    println!();

    // ========== 保持连接一段时间 ==========
    println!("--- Keeping connection alive for 3 seconds ---");
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Connected: {}", client.is_connected().await);
    println!();

    // ========== 断开连接 ==========
    println!("--- Disconnecting ---");
    match client.disconnect().await {
        Ok(()) => {
            println!("✅ Disconnected from IoTDB");
            println!("Connected: {}", client.is_connected().await);
        }
        Err(e) => {
            println!("❌ Failed to disconnect: {}", e);
        }
    }
    println!();

    // ========== 测试连接后状态 ==========
    println!("--- After Disconnection ---");
    println!("Connected: {}", client.is_connected().await);
    println!();

    // ========== 尝试在断开后执行操作 ==========
    println!("--- Testing operation after disconnection ---");
    match client.query("SHOW DATABASES").await {
        Ok(_) => println!("✅ Query succeeded (shouldn't happen)"),
        Err(e) => println!("❌ Query failed as expected: {}", e),
    }
    println!();

    // ========== 重新连接测试 ==========
    println!("--- Testing reconnection ---");
    match client.connect().await {
        Ok(()) => {
            println!("✅ Reconnected successfully");
            println!("Connected: {}", client.is_connected().await);
            client.disconnect().await?;
            println!("✅ Disconnected again");
        }
        Err(e) => {
            println!("❌ Failed to reconnect: {}", e);
        }
    }
    println!();

    println!("✅ Connection test completed successfully!");

    Ok(())
}