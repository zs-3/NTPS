use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::{config::Config, identity};

#[derive(Serialize)]
struct NodeRequest {
    node_id: String, public_key: String, signature: String, software_version: String,
}
#[derive(Serialize)]
struct DomainRequest {
    domain: String, node_id: String, public_key: String, signature: String,
}
#[derive(Deserialize)]
struct ApiError { error: Option<String>, message: Option<String> }

fn client(cfg: &Config) -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder().https_only(false).timeout(std::time::Duration::from_secs(15)).build()?)
}
fn sign_payload(key: &SigningKey, payload: &str) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(key.sign(payload.as_bytes()).to_bytes())
}
pub async fn register_node(cfg: &Config, key: &SigningKey) -> Result<serde_json::Value> {
    let public = identity::public_key_b64(key);
    let node_id = identity::public_id(&cfg.identity_file)?;
    let payload = format!("NTPS-NODE-V1\n{}\n{}", node_id, public);
    let req = NodeRequest { node_id, public_key: public, signature: sign_payload(key, &payload), software_version: env!("CARGO_PKG_VERSION").into() };
    post(cfg, "/api/v1/nodes/register", &req).await
}
pub async fn register_domain(cfg: &Config, key: &SigningKey, domain: &str) -> Result<serde_json::Value> {
    validate_domain(domain)?;
    let public = identity::public_key_b64(key);
    let node_id = identity::public_id(&cfg.identity_file)?;
    let payload = format!("NTPS-DOMAIN-V1\n{}\n{}", domain.to_ascii_lowercase(), node_id);
    let req = DomainRequest { domain: domain.to_ascii_lowercase(), node_id, public_key: public, signature: sign_payload(key, &payload) };
    post(cfg, "/api/v1/domains/register", &req).await
}
pub async fn resolve(cfg: &Config, domain: &str) -> Result<serde_json::Value> {
    validate_domain(domain)?;
    let url = format!("{}/api/v1/domains/resolve?name={}", cfg.registry_url.trim_end_matches('/'), url_encode(domain));
    let r = client(cfg)?.get(url).send().await?;
    let status = r.status();
    let body: serde_json::Value = r.json().await?;
    if !status.is_success() { bail!("{}", body["error"].as_str().unwrap_or("registry request failed")); }
    Ok(body)
}
async fn post<T: Serialize>(cfg: &Config, path: &str, body: &T) -> Result<serde_json::Value> {
    let mut req = client(cfg)?.post(format!("{}{}", cfg.registry_url.trim_end_matches('/'), path)).json(body);
    if !cfg.registry_token.is_empty() { req = req.bearer_auth(&cfg.registry_token); }
    let r = req.send().await.context("connecting to registry")?;
    let status = r.status();
    let value: serde_json::Value = r.json().await.context("registry returned invalid JSON")?;
    if !status.is_success() { bail!("{}", value["error"].as_str().or(value["message"].as_str()).unwrap_or("registry rejected request")); }
    Ok(value)
}
fn validate_domain(d: &str) -> Result<()> {
    let d = d.trim_end_matches('.').to_ascii_lowercase();
    if !d.ends_with(".ntp") || d.len() > 253 || d.len() < 5 { bail!("domain must be a valid name under .ntp"); }
    let label = &d[..d.len()-4];
    if label.split('.').any(|x| x.is_empty() || x.len() > 63 || !x.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') || x.starts_with('-') || x.ends_with('-')) { bail!("invalid .ntp domain label"); }
    Ok(())
}
fn url_encode(s: &str) -> String { s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-._".contains(&b) { (b as char).to_string() } else { format!("%{:02X}", b) }).collect() }
