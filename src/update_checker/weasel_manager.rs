use std::{
    os::windows::process::CommandExt, path::PathBuf, process::Command, thread, time::Duration,
};

pub struct WeaselManager {
    weasel_root: PathBuf,
}

impl WeaselManager {
    pub fn new(weasel_path: &PathBuf) -> Self {
        Self {
            weasel_root: weasel_path.clone(),
        }
    }

    /// 部署小狼毫
    pub fn deploy(&self) -> bool {
        println!("正在部署小狼毫...");

        let deployer_path = self.weasel_root.join("WeaselDeployer.exe");
        if !deployer_path.exists() {
            eprintln!("❌ 未找到WeaselDeployer.exe: {:?}", deployer_path);
            return false;
        }

        // 首先停止服务
        self.stop_weasel_service();

        // 等待一段时间确保服务完全停止
        thread::sleep(Duration::from_secs(2));

        // 执行部署
        let output = Command::new(&deployer_path).arg("/deploy").status();

        match output {
            Ok(status) => {
                if status.success() {
                    println!("✅ 小狼毫部署成功");

                    // 等待部署完成后重启服务
                    thread::sleep(Duration::from_secs(1));
                    self.start_weasel_service();

                    true
                } else {
                    eprintln!("❌ 小狼毫部署失败，状态码: {}", status);
                    false
                }
            }
            Err(e) => {
                eprintln!("❌ 执行部署命令失败: {}", e);
                false
            }
        }
    }

    /// 停止小狼毫服务
    fn stop_weasel_service(&self) {
        println!("正在停止小狼毫服务...");

        let server_path = self.weasel_root.join("WeaselServer.exe");
        if server_path.exists() {
            let _ = Command::new(&server_path).arg("/q").status();
        }

        // 使用taskkill强制结束进程
        let _ = Command::new("taskkill")
            .args(&["/f", "/im", "WeaselServer.exe"])
            .creation_flags(0x08000000)
            .output();

        let _ = Command::new("taskkill")
            .args(&["/f", "/im", "WeaselDeployer.exe"])
            .creation_flags(0x08000000)
            .output();
    }

    /// 启动小狼毫服务
    fn start_weasel_service(&self) {
        println!("正在启动小狼毫服务...");

        let server_path = self.weasel_root.join("WeaselServer.exe");
        if server_path.exists() {
            let _ = Command::new(&server_path).spawn();
        }
    }
}
