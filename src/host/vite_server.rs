use std::process::{Child, Command};
use which::which;

pub struct ViteServer {
    pub process: Child,
    pub port: u16,
}

impl ViteServer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let npm = which("npm").expect("npm not found in PATH");
        let port = portpicker::pick_unused_port().expect("no free ports");

        let process = Command::new(npm)
            .args([
                "run",
                "dev",
                "--",
                "--port",
                &port.to_string(),
                "--strictPort",
                "--host",
            ])
            .current_dir("ui")
            .spawn()?;

        Ok(Self { process, port })
    }
}

impl Drop for ViteServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
