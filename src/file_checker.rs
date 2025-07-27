use std::{path::Path, process::Command};

/// 使用系统自带的 PowerShell 计算文件的 SHA3-256 哈希值
pub fn calculate_sha3_256(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    println!("🔍 正在计算文件 SHA3-256 校验和...");

    // 使用 PowerShell 的 Get-FileHash 命令计算 SHA3-256
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "[System.Console]::OutputEncoding = [System.Console]::InputEncoding = [System.Text.Encoding]::UTF8;",
            "-Command",
            &format!(
                "[System.Console]::OutputEncoding = [System.Console]::InputEncoding = [System.Text.Encoding]::UTF8;Get-FileHash -Path '{}' -Algorithm SHA3-256 | Select-Object -ExpandProperty Hash",
                file_path.display()
            ),
        ])
        .output()?;

    if output.status.success() {
        let hash = String::from_utf8(output.stdout)?.trim().to_lowercase();

        if hash.is_empty() {
            return Err("PowerShell 返回空的哈希值".into());
        }

        println!("✅ 文件 SHA3-256: {}", hash);
        Ok(hash)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell 计算哈希失败: {}", error).into())
    }
}

/// 验证文件的 SHA3-256 校验和
pub fn verify_sha3_256(
    file_path: &Path,
    expected_hash: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let calculated_hash = calculate_sha3_256(file_path)?;
    let expected_hash_lower = expected_hash.to_lowercase();

    let is_valid = calculated_hash == expected_hash_lower;

    if is_valid {
        println!("✅ 文件校验成功");
    } else {
        println!("❌ 文件校验失败!");
        println!("  期望: {}", expected_hash_lower);
        println!("  实际: {}", calculated_hash);
    }

    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_calculate_sha3_256() {
        // 创建一个临时文件用于测试
        let temp_file = std::env::temp_dir().join("test_sha3_256.txt");
        let mut file = fs::File::create(&temp_file).unwrap();
        writeln!(file, "Hello, World!").unwrap();

        // 计算哈希值
        let result = calculate_sha3_256(&temp_file);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert_eq!(hash.len(), 64); // SHA3-256 应该是 64 个十六进制字符

        // 清理
        fs::remove_file(&temp_file).ok();
    }
}
