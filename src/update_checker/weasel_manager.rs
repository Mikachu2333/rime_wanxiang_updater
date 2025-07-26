use std::{path::PathBuf, process::Command, thread, time::Duration};

pub struct WeaselManager {
    weasel_path: PathBuf,
}

impl WeaselManager {
    pub fn new(weasel_path: PathBuf) -> Self {
        Self { weasel_path }
    }

    /// 部署小狼毫
    pub fn deploy(&self, weasel_path: &PathBuf) -> bool {
        println!("正在部署小狼毫...");

        let weasel_deployer = weasel_path.join("WeaselDeployer.exe");
        if !weasel_deployer.exists() {
            eprintln!("❌ 未找到 WeaselDeployer.exe: {:?}", weasel_deployer);
            return false;
        }

        let output = Command::new(&weasel_deployer)
            .arg("/deploy")
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ 小狼毫部署完成");
                    true
                } else {
                    eprintln!("❌ 小狼毫部署失败: {}", String::from_utf8_lossy(&result.stderr));
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 执行部署命令失败: {}", e);
                false
            }
        }
    }

    /// 重启小狼毫服务
    pub fn restart_service(&self) -> bool {
        println!("正在重启小狼毫服务...");

        let weasel_server = self.weasel_path.join("WeaselServer.exe");
        if !weasel_server.exists() {
            eprintln!("❌ 未找到 WeaselServer.exe: {:?}", weasel_server);
            return false;
        }

        // 先停止服务
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "WeaselServer.exe"])
            .output();

        // 等待一下再启动
        thread::sleep(Duration::from_millis(1000));

        // 启动服务
        match Command::new(&weasel_server).spawn() {
            Ok(_) => {
                println!("✅ 小狼毫服务重启完成");
                true
            }
            Err(e) => {
                eprintln!("❌ 启动小狼毫服务失败: {}", e);
                false
            }
        }
    }
}