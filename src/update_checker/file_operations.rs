use std::{fs, path::PathBuf, process::Command};

pub struct FileOperations {
    weasel_path: PathBuf,
}

impl FileOperations {
    pub fn new(weasel_path: PathBuf) -> Self {
        Self { weasel_path }
    }

    /// 下载文件
    pub fn download_file(&self, curl_path: &PathBuf, url: &str, save_path: &PathBuf) -> bool {
        println!("正在下载: {}", url);
        
        // 确保目标目录存在
        if let Some(parent) = save_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("创建目录失败: {}", e);
                return false;
            }
        }

        let output = Command::new(curl_path)
            .args(&[
                "-L", "-o", 
                save_path.to_str().unwrap(),
                "--progress-bar",
                "--fail",
                url
            ])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ 下载完成: {}", save_path.display());
                    true
                } else {
                    eprintln!("❌ 下载失败: {}", String::from_utf8_lossy(&result.stderr));
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 下载命令执行失败: {}", e);
                false
            }
        }
    }

    /// 验证SHA256哈希值
    pub fn verify_sha256(&self, file_path: &PathBuf, expected_hash: &str) -> bool {
        if expected_hash.is_empty() {
            return true; // 如果没有提供哈希值，跳过验证
        }

        println!("正在验证文件哈希...");

        // 使用系统的certutil命令计算SHA256
        let output = Command::new("certutil")
            .args(&["-hashfile", file_path.to_str().unwrap(), "SHA256"])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let output_str = String::from_utf8_lossy(&result.stdout);
                    // certutil输出格式: 第二行是哈希值
                    if let Some(hash_line) = output_str.lines().nth(1) {
                        let actual_hash = hash_line.trim().replace(" ", "");
                        
                        if actual_hash.eq_ignore_ascii_case(expected_hash) {
                            println!("✅ 文件哈希验证通过");
                            true
                        } else {
                            eprintln!("❌ 文件哈希验证失败");
                            eprintln!("  期望: {}", expected_hash);
                            eprintln!("  实际: {}", actual_hash);
                            false
                        }
                    } else {
                        eprintln!("❌ 无法解析哈希值输出");
                        false
                    }
                } else {
                    eprintln!("❌ 计算哈希值失败: {}", String::from_utf8_lossy(&result.stderr));
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 执行certutil命令失败: {}", e);
                false
            }
        }
    }

    /// 解压ZIP文件使用7z.exe
    pub fn extract_zip(&self, zip_path: &PathBuf, extract_path: &PathBuf) -> bool {
        println!("正在解压: {} -> {}", zip_path.display(), extract_path.display());
        
        let seven_zip = self.weasel_path.join("7z.exe");
        if !seven_zip.exists() {
            eprintln!("❌ 未找到 7z.exe: {:?}", seven_zip);
            eprintln!("   请确保小狼毫安装目录中包含 7z.exe");
            return false;
        }

        // 确保解压目录存在
        if let Err(e) = fs::create_dir_all(extract_path) {
            eprintln!("❌ 创建解压目录失败: {}", e);
            return false;
        }

        // 使用7z解压文件
        let output = Command::new(&seven_zip)
            .args(&[
                "x",                                    // 解压命令
                zip_path.to_str().unwrap(),            // 源文件
                "-o".to_owned() + extract_path.to_str().unwrap(), // 输出目录
                "-y"                                    // 自动覆盖
            ])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ 解压完成");
                    true
                } else {
                    eprintln!("❌ 解压失败: {}", String::from_utf8_lossy(&result.stderr));
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 执行7z命令失败: {}", e);
                false
            }
        }
    }
}