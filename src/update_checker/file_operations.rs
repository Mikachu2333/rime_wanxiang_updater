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
    pub fn download_file(&self, curl_path: &PathBuf, url: &str, save_path: &PathBuf, github_cookie: Option<String>) -> bool {
        println!("正在下载: {}", url);

        // 如果文件已存在，先删除
        if save_path.exists() {
            if let Err(e) = std::fs::remove_file(save_path) {
                eprintln!("❌ 无法删除旧文件: {}", e);
                return false;
            }
        }

        // 使用 spawn 和 wait 来实现实时进度显示
        let mut command = Command::new(curl_path);
        command.args(&[
                "-C",
                "-",              // 断点续传
                "-L",             // 跟随重定向
                "--progress-bar", // 显示进度条
                "--fail",         // 在HTTP错误时失败
                "--connect-timeout",
                "30", // 连接超时
                "--max-time",
                "1800", // 最大下载时间(30分钟)
                "-o",   // 输出文件
            ])
            .arg(save_path)
            .arg(url);

        // 如果提供了cookie，添加cookie参数
        if let Some(cookie) = github_cookie {
            command.arg("-H").arg(format!("Cookie: {}", cookie));
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("❌ 执行curl命令失败: {}", e);
                return false;
            }
        };

        // 等待下载完成
        let result = match child.wait() {
            Ok(status) => status,
            Err(e) => {
                eprintln!("❌ 等待curl进程失败: {}", e);
                return false;
            }
        };

        if result.success() {
            // 验证文件是否确实下载完成
            if save_path.exists() {
                let file_size = std::fs::metadata(save_path).map(|m| m.len()).unwrap_or(0);

                if file_size > 1000 {
                    // 至少1KB，避免下载失败的小文件
                    println!("✅ 下载完成: {:?} ({} bytes)", save_path, file_size);
                    true
                } else {
                    eprintln!("❌ 下载的文件太小，可能下载失败: {} bytes", file_size);
                    // 清理不完整的文件
                    let _ = std::fs::remove_file(save_path);
                    false
                }
            } else {
                eprintln!("❌ 下载后文件不存在");
                false
            }
        } else {
            eprintln!("❌ 下载失败，curl退出码: {}", result);
            false
        }
    }

    /// 解压ZIP文件
    pub fn extract_zip(&self, zip_path: &PathBuf, extract_path: &PathBuf) -> bool {
        println!("正在解压文件...");

        let output = Command::new(&self.zip_path)
            .args(&["x"]) // 解压命令
            .arg(zip_path) // 源文件
            .arg(&format!("-o{}", extract_path.display())) // 输出目录
            .arg("-y") // 覆盖确认
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
