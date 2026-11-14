
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const MAGIC: &[u8; 8] = b"NTPS01\r\n";

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub host: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub content_length: u64,
    pub body_sha256: String,
}

pub async fn read_request<R: AsyncReadExt + Unpin>(
    r: &mut R,
    max: usize,
) -> Result<Request> {
    let mut magic = [0u8; 8];

    r.read_exact(&mut magic).await?;

    if &magic != MAGIC {
        bail!("invalid NTPS frame magic");
    }

    let len = r.read_u32().await? as usize;

    if len == 0 || len > max {
        bail!("request header too large");
    }

    let mut b = vec![0; len];
    r.read_exact(&mut b).await?;

    Ok(serde_json::from_slice(&b)?)
}

pub async fn write_response<W: AsyncWriteExt + Unpin>(
    w: &mut W,
    meta: &Response,
    body: &[u8],
) -> Result<()> {
    let b = serde_json::to_vec(meta)?;

    w.write_all(MAGIC).await?;
    w.write_u32(b.len() as u32).await?;
    w.write_all(&b).await?;
    w.write_all(body).await?;
    w.shutdown().await?;

    Ok(())
}
