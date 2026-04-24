use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use colored::*;

/// Fetches the Stainless stats file and extracts the openapi_spec_url
async fn get_stainless_spec_url(owner: &str, repo: &str) -> anyhow::Result<String> {
    let stats_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/.stats.yml",
        owner, repo
    );
    let response = reqwest::get(&stats_url).await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to fetch .stats.yml: HTTP {}", response.status()));
    }
    let content = response.text().await?;
    for line in content.lines() {
        if line.starts_with("openapi_spec_url:") {
            let url = line.splitn(2, ':').nth(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid openapi_spec_url line"))?
                .trim()
                .to_string();
            return Ok(url);
        }
    }
    Err(anyhow::anyhow!("openapi_spec_url not found in .stats.yml"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let openai_url = get_stainless_spec_url("openai", "openai-python").await
        .unwrap_or_else(|_| "https://storage.googleapis.com/stainless-sdk-openapi-specs/openai/openai-openapi-*.yml".to_string());

    let providers = vec![
        ("openai", openai_url.as_str()),
        // ("anthropic", "https://storage.googleapis.com/stainless-sdk-openapi-specs/anthropic/anthropic-6811827071199207cf69c4e64cae1b41e7e74cdb14048e9c748701e474c694a7.yml"),
    ];

    fs::create_dir_all("specs")?;

    for (name, url) in providers {
        // Check both possible extensions for existing file
        let json_path = format!("specs/{}.json", name);
        let yaml_path = format!("specs/{}.yaml", name);

        let (local_hash, _existing_path) = if Path::new(&json_path).exists() {
            let bytes = fs::read(&json_path)?;
            (hash_bytes(&bytes), Some(json_path))
        } else if Path::new(&yaml_path).exists() {
            let bytes = fs::read(&yaml_path)?;
            (hash_bytes(&bytes), Some(yaml_path))
        } else {
            println!("[INFO] {}: No local spec found, will download.", name.yellow());
            (String::new(), None)
        };

        // 2. Fetch Remote (Streamed to avoid memory spikes)
        println!("[AUDIT] Checking {}...", name.cyan());
        let remote_response = reqwest::get(url).await?;
        if !remote_response.status().is_success() {
            println!(
                "[ERROR] {}: HTTP {} - {}",
                name.red(),
                remote_response.status().as_u16(),
                remote_response.status().canonical_reason().unwrap_or("Unknown")
            );
            continue;
        }
        let remote_bytes = remote_response.bytes().await?;
        let remote_hash = hash_bytes(&remote_bytes);

        // Detect format and set appropriate extension
        let content = String::from_utf8_lossy(&remote_bytes);
        let ext = if content.trim_start().starts_with('{') { "json" } else { "yaml" };
        let local_path = format!("specs/{}.{}", name, ext);

        // 3. Compare and Notify
        if local_hash.is_empty() {
            println!("[NEW] {}: Downloading initial spec.", name.green());
            fs::write(&local_path, &remote_bytes)?;
        } else if local_hash != remote_hash {
            println!(
                "[WARN] {}: Schema change detected on remote!",
                name.yellow().bold()
            );
            println!("   Local:  {}", &local_hash[..12.min(local_hash.len())].dimmed());
            println!("   Remote: {}", &remote_hash[..12].green());
            println!("   Action: Run {} to update.", "scripts/update_specs.sh".bold());
        } else {
            println!("[OK] {}: Up to date.", name.green());
        }
    }
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
