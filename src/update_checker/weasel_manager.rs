use std::{path::PathBuf, process::Command, thread, time::Duration};

pub struct WeaselManager {
    weasel_path: PathBuf,
}

impl WeaselManager {
    pub fn new(weasel_path: PathBuf) -> Self {
        Self { weasel_path }
    }

    /// 终止小狼毫进程
    pub fn terminate_processes(&self) {
        println!("正在终止小狼毫进程...");

        let server_exe = self.weasel_path.join("WeaselServer.exe");
        if server_exe.exists() {
            let _ = Command::new(server_exe).arg("/q").output();
        }

        thread::sleep(Duration::from_millis(500));

        let _ = Command::new("taskkill")
            .args(["/IM", "WeaselServer.exe", "/F"])
            .output();

        let _ = Command::new("taskkill")
            .args(["/IM", "WeaselDeployer.exe", "/F"])
            .output();

        println!("进程终止完成");
    }

    /// 部署小狼毫
    pub fn deploy(&self) -> bool {
        println!("正在部署小狼毫...");

        let server_exe = self.weasel_path.join("WeaselServer.exe");
        if server_exe.exists() {
            let _ = Command::new(&server_exe).spawn();
            thread::sleep(Duration::from_secs(2));
        }

        let deployer_exe = self.weasel_path.join("WeaselDeployer.exe");
        if deployer_exe.exists() {
            let output = Command::new(deployer_exe)
                .arg("/deploy")
                .output()
                .expect("部署器执行失败");

            if output.status.success() {
                println!("✅ 部署成功");
                true
            } else {
                eprintln!("❌ 部署失败: {}", String::from_utf8_lossy(&output.stderr));
                false
            }
        } else {
            eprintln!("❌ 未找到部署器");
            false
        }
    }
}