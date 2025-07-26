use std::{path::PathBuf, process::Command};

pub struct FileOperations {
    zip_path: PathBuf,
}

impl FileOperations {
    pub fn new(zip_path: &PathBuf) -> Self {
        Self {
            zip_path: zip_path.clone(),
        }
    }

    /// 下载文件
    pub fn download_file(&self, curl_path: &PathBuf, url: &str, save_path: &PathBuf) -> bool {
        println!("正在下载: {}", url);
        
        let output = Command::new(curl_path)
            .args(&[
                "-L",           // 跟随重定向
                "--progress-bar", // 显示进度条
                "-o",           // 输出文件
            ])
            .arg(save_path)
            .arg(url)
            .status();

        match output {
            Ok(status) => {
                if status.success() {
                    println!("✅ 下载完成: {:?}", save_path);
                    true
                } else {
                    eprintln!("❌ 下载失败，状态码: {}", status);
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 下载失败: {}", e);
                false
            }
        }
    }

    /// 验证SHA256哈希
    pub fn verify_sha256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        println!("正在验证文件完整性...");
        
        let output = Command::new("certutil")
            .args(&["-hashfile"])
            .arg(file_path)
            .arg("SHA256")
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let output_str = String::from_utf8_lossy(&result.stdout);
                    // certutil 输出格式：第二行包含哈希值
                    if let Some(hash_line) = output_str.lines().nth(1) {
                        let actual_hash = hash_line.trim().replace(" ", "").to_lowercase();
                        let expected_hash = expected_hash.trim().replace(" ", "").to_lowercase();
                        
                        if actual_hash == expected_hash {
                            println!("✅ 文件完整性验证通过");
                            return true;
                        } else {
                            eprintln!("❌ 文件完整性验证失败");
                            eprintln!("  期望: {}", expected_hash);
                            eprintln!("  实际: {}", actual_hash);
                        }
                    }
                } else {
                    eprintln!("❌ 计算SHA256失败: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                eprintln!("❌ 执行certutil命令失败: {}", e);
            }
        }

        false
    }

    /// 解压ZIP文件
    pub fn extract_zip(&self, zip_path: &PathBuf, extract_path: &PathBuf) -> bool {
        println!("正在解压文件...");
        
        let output = Command::new(&self.zip_path)
            .args(&["x"])           // 解压命令
            .arg(zip_path)          // 源文件
            .arg(&format!("-o{}", extract_path.display())) // 输出目录
            .arg("-y")              // 覆盖确认
            .status();

        match output {
            Ok(status) => {
                if status.success() {
                    println!("✅ 解压完成: {:?}", extract_path);
                    true
                } else {
                    eprintln!("❌ 解压失败，状态码: {}", status);
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 解压失败: {}", e);
                eprintln!("   请确保7z.exe存在于小狼毫安装目录中");
                false
            }
        }
    }
}