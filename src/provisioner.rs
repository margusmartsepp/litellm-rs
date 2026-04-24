use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use anyhow::Context;

pub struct Provisioner {
    process: Option<Child>,
    port: u16,
}

impl Provisioner {
    pub fn new(port: u16) -> Self {
        Self { process: None, port }
    }

    /// Spawns the Bifrost gateway using npx.
    /// Ensures that the process is piped to NULL to maintain performance.
    pub async fn spawn(&mut self) -> anyhow::Result<()> {
        let child = Command::new("npx")
            .args([
                "@helicone/bifrost",
                "-p",
                &self.port.to_string(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn Bifrost via npx. Ensure Node.js is installed.")?;

        self.process = Some(child);

        // Wait for the gateway to bind to the port
        self.wait_for_ready().await
    }

    async fn wait_for_ready(&self) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let health_url = format!("http://localhost:{}/health", self.port);

        for _ in 0..10 {
            if let Ok(res) = client.get(&health_url).send().await {
                if res.status().is_success() {
                    return Ok(());
                }
            }
            sleep(Duration::from_millis(500)).await;
        }

        Err(anyhow::anyhow!("Bifrost failed to start within 5 seconds"))
    }
}

impl Drop for Provisioner {
    /// Ensures no zombie processes remain after the Rust client exits.
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
        }
    }
}
