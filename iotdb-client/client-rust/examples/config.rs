// examples/config.rs
use iotdb_rust_client::{Client, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ========== 显示客户端版本 ==========
    println!("╔════════════════════════════════════════════╗");
    println!("║     IoTDB Rust Client v{}            ║", iotdb_rust_client::VERSION);
    println!("╚════════════════════════════════════════════╝");
    println!();

    // ========== 方式1：使用默认配置 ==========
    println!("--- Using Default Configuration ---");
    let client = Client::new("localhost", 6667);

    println!("  Host: {}", client.config.host);
    println!("  Port: {}", client.config.port);
    println!("  Timeout: {}s", client.config.timeout_secs);
    println!("  Session Timeout: {}s", client.config.session_timeout_secs);
    println!("  TLS Enabled: {}", client.config.enable_tls);
    println!("  Authentication: {}", if client.config.username.is_some() { "Enabled" } else { "Disabled" });

    // 连接
    match client.connect().await {
        Ok(()) => {
            println!("  Connection Status: ✅ Connected");

            // 获取 IoTDB 版本
            match client.version().await {
                Ok(version) => println!("  IoTDB Version: {}", version),
                Err(e) => println!("  IoTDB Version: Failed to query ({})", e),
            }

            // 显示数据库当前模式
            match client.show_current_sql_dialect().await {
                Ok(result) => {
                    println!("\n--- Sql Dialect ---");
                    for row in &result.rows {
                        for (key, value) in &row.fields {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
                Err(e) => println!("  SQL DIALECT: Failed to query ({})", e),
            }

            // 显示数据库列表
            match client.show_databases().await {
                Ok(result) => {
                    println!("\n--- Databases ---");
                    if result.rows.is_empty() {
                        println!("  No databases found");
                    } else {
                        for row in &result.rows {
                            for (key, value) in &row.fields {
                                println!("  {}: {}", key, value);
                            }
                        }
                    }
                }
                Err(e) => println!("  Show Databases: Failed to query ({})", e),
            }

            client.disconnect().await?;
            println!("\n  ✅ Disconnected");
        }
        Err(e) => {
            println!("  Connection Status: ❌ Failed");
            println!("  Error: {:?}", e); // 👈 🔥 这里也改成 {:?}
        }
    }

    println!();

    // ========== 方式2：使用自定义配置 ==========
    println!("--- Using Custom Configuration ---");
    let config = ClientConfig::new("127.0.0.1", 6667)
        .with_timeout(60)
        .with_session_timeout(7200)
        .with_auth("root", "root");

    let client = Client::with_config(config);

    println!("  Host: {}", client.config.host);
    println!("  Port: {}", client.config.port);
    println!("  Timeout: {}s", client.config.timeout_secs);
    println!("  Session Timeout: {}s", client.config.session_timeout_secs);
    println!("  TLS Enabled: {}", client.config.enable_tls);
    println!("  Authentication: {}", if client.config.username.is_some() { "✅ Enabled" } else { "❌ Disabled" });

    if let Some(username) = &client.config.username {
        println!("  Username: {}", username);
    }

    // 尝试连接（如果认证信息正确）
    match client.connect().await {
        Ok(()) => {
            println!("  Connection Status: ✅ Connected");
            client.disconnect().await?;
            println!("  ✅ Disconnected");
        }
        Err(e) => {
            println!("  Connection Status: ❌ Failed");
            println!("  Error: {:?}", e); // 👈 🔥 这里也改成 {:?}
        }
    }

    println!();

    // ========== 配置说明 ==========
    println!("--- Configuration Notes ---");
    println!("  • Default port: 6667");
    println!("  • Default timeout: 30s");
    println!("  • Default session timeout: 3600s (1 hour)");
    println!("  • TLS support: Not yet implemented");
    println!("  • Authentication: Username/Password");

    println!("\n✅ Example completed successfully!");

    Ok(())
}