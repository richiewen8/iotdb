use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let iotdb_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."));
    
    let out_dir = manifest_dir.join("src/thrift");
    fs::create_dir_all(&out_dir).unwrap();
    
    // 检查Thrift编译器
    let thrift_check = Command::new("thrift")
        .arg("--version")
        .output();
    
    if let Err(e) = thrift_check {
        println!("cargo:warning=Thrift compiler not found: {}", e);
        return;
    }
    
    // 定义所有thrift文件及其位置
    let thrift_files = vec![
        (iotdb_root.join("iotdb-protocol/thrift-commons/src/main/thrift"), "common.thrift"),
        (iotdb_root.join("iotdb-protocol/thrift-datanode/src/main/thrift"), "client.thrift"),
        (iotdb_root.join("iotdb-protocol/thrift-datanode/src/main/thrift"), "datanode.thrift"),
        (iotdb_root.join("iotdb-protocol/thrift-confignode/src/main/thrift"), "confignode.thrift"),
    ];
    
    let mut generated_mods = Vec::new();
    
    for (dir, file_name) in &thrift_files {
        let thrift_path = dir.join(file_name);
        if !thrift_path.exists() {
            println!("cargo:warning=File not found: {}", thrift_path.display());
            continue;
        }
        
        println!("cargo:rerun-if-changed={}", thrift_path.display());
        
        let status = Command::new("thrift")
            .arg("--gen")
            .arg("rs")
            .arg("-I")
            .arg(dir)
            .arg("-I")
            .arg(iotdb_root.join("iotdb-protocol/thrift-commons/src/main/thrift"))
            .arg("-o")
            .arg(&out_dir)
            .arg(&thrift_path)
            .status();
        
        if let Ok(s) = status {
            if s.success() {
                let mod_name = file_name.replace(".thrift", "");
                generated_mods.push(mod_name);
                println!("cargo:info=Generated: {}", file_name);
            } else {
                println!("cargo:warning=Failed to generate: {}", file_name);
            }
        } else {
            println!("cargo:warning=Error running thrift for: {}", file_name);
        }
    }
    
    // 创建 mod.rs
    if !generated_mods.is_empty() {
        let mod_content = generated_mods
            .iter()
            .map(|m| format!("pub mod {};", m))
            .collect::<Vec<_>>()
            .join("\n");
        
        let mod_rs_path = out_dir.join("mod.rs");
        fs::write(&mod_rs_path, mod_content).unwrap();
        println!("cargo:info=Created mod.rs with {} modules", generated_mods.len());
    } else {
        let mod_rs_path = out_dir.join("mod.rs");
        fs::write(&mod_rs_path, "// No thrift files generated\n").unwrap();
        println!("cargo:warning=No thrift files were generated");
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}