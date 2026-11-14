use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{net::SocketAddr, path::{Component, Path, PathBuf}, sync::Arc};
use tokio::{net::{TcpListener, TcpStream}, io::AsyncWriteExt};
use tracing::{info, warn};
use crate::wire::{self, Request, Response};

pub async fn run(root: PathBuf, addr: SocketAddr, max_bytes: u64) -> Result<()> {
    let listener = TcpListener::bind(addr).await.context("binding NTPS listener")?;
    let root = Arc::new(root);
    loop {
        let (stream, peer) = listener.accept().await?;
        let root = root.clone();
        tokio::spawn(async move {
            if let Err(e) = handle(stream, root, max_bytes).await { warn!(%peer, error=%e, "request failed"); }
        });
    }
}
async fn handle(mut stream: TcpStream, root: Arc<PathBuf>, max_bytes: u64) -> Result<()> {
    let req = wire::read_request(&mut stream, 16_384).await?;
    if req.method != "GET" && req.method != "HEAD" {
        return wire::write_response(&mut stream, &Response { status:405, content_type:"text/plain".into(), content_length:0, body_sha256:String::new() }, &[]).await;
    }
    let path = safe_path(&root, &req.path)?;
    let path = if path.is_dir() { path.join("index.html") } else { path };
    let canonical = match tokio::fs::canonicalize(&path).await {
        Ok(p) => p,
        Err(_) => return wire::write_response(&mut stream, &Response { status:404, content_type:"text/plain".into(), content_length:0, body_sha256:String::new() }, &[]).await,
    };
    if !canonical.starts_with(root.as_path()) {
        return wire::write_response(&mut stream, &Response { status:403, content_type:"text/plain".into(), content_length:0, body_sha256:String::new() }, &[]).await;
    }
    let meta = tokio::fs::metadata(&canonical).await?;
    if !meta.is_file() || meta.len() > max_bytes {
        return wire::write_response(&mut stream, &Response { status:413, content_type:"text/plain".into(), content_length:0, body_sha256:String::new() }, &[]).await;
    }
    let body = if req.method == "HEAD" { Vec::new() } else { tokio::fs::read(&canonical).await? };
    let hash = hex::encode(Sha256::digest(&body));
    let mime = mime_guess::from_path(&canonical).first_or_octet_stream().to_string();
    let length = if req.method == "HEAD" { meta.len() } else { body.len() as u64 };
    wire::write_response(&mut stream, &Response { status:200, content_type:mime, content_length:length, body_sha256:hash }, &body).await
}
fn safe_path(root: &Path, raw: &str) -> Result<PathBuf> {
    let decoded = raw.split('?').next().unwrap_or("/");
    let decoded = decoded.trim_start_matches('/');
    let mut out = root.to_path_buf();
    for c in Path::new(decoded).components() {
        match c { Component::Normal(s) => out.push(s), Component::CurDir => {}, _ => anyhow::bail!("unsafe path") }
    }
    Ok(out)
}
