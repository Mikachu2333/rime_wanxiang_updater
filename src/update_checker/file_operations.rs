use std::{path::PathBuf, process::Command};

pub struct FileOperations;

impl FileOperations {
    pub fn new() -> Self {
        Self
    }

    /// 下载文件
    pub fn download_file(&self, curl_path: &PathBuf, url: &str, save_path: &PathBuf) -> bool {
        println!("正在下载: {}", url);

        let output = Command::new(curl_path)
            .args([
                "-L",
                "-o",
                save_path.to_str().unwrap(),
                "--create-dirs",
                "--progress-bar",
                url,
            ])
            .output()
            .expect("curl 下载失败");

        if output.status.success() {
            println!("下载完成: {:?}", save_path);
            true
        } else {
            eprintln!("下载失败: {}", String::from_utf8_lossy(&output.stderr));
            false
        }
    }

    /// 验证SHA256哈希
    pub fn verify_sha256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        if expected_hash.is_empty() {
            println!("跳过哈希验证（未提供期望值）");
            return true;
        }

        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Get-FileHash -Path '{}' -Algorithm SHA256 | Select-Object -ExpandProperty Hash",
                    file_path.display()
                ),
            ])
            .output()
            .expect("PowerShell 哈希计算失败");

        if output.status.success() {
            let calculated_hash = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            let expected_hash = expected_hash.to_lowercase();

            calculated_hash == expected_hash
        } else {
            eprintln!("哈希计算失败: {}", String::from_utf8_lossy(&output.stderr));
            false
        }
    }

    /// 解压ZIP文件
    pub fn extract_zip(&self, zip_path: &PathBuf, extract_to: &PathBuf) -> bool {
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    extract_to.display()
                ),
            ])
            .output()
            .expect("PowerShell 解压失败");

        output.status.success()
    }

    /// 格式化文件大小
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }
}