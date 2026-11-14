
let body = if req.method == "HEAD" {
    Vec::new()
} else {
    tokio::fs::read(&canonical).await?
};

let hash = hex::encode(Sha256::digest(&body));

let mime = mime_guess::from_path(&canonical)
    .first_or_octet_stream()
    .to_string();

let length = if req.method == "HEAD" {
    meta.len()
} else {
    body.len() as u64
};

wire::write_response(
    &mut stream,
    &Response {
        status: 200,
        content_type: mime,
        content_length: length,
        body_sha256: hash,
    },
    &body,
)
.await
