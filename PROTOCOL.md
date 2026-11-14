# NTPS/0.1 wire format

This MVP defines a deliberately simple framed TCP exchange. It is not the final encrypted transport.

## Request

1. ASCII magic `NTPS01\r\n` (8 bytes)
2. unsigned 32-bit big-endian JSON header length
3. UTF-8 JSON request object (`method`, `host`, `path`)

## Response

1. Same 8-byte magic
2. unsigned 32-bit big-endian JSON metadata length
3. UTF-8 JSON metadata (`status`, `content_type`, `content_length`, `body_sha256`)
4. Exactly `content_length` body bytes

One connection carries one request/response. A production implementation must move this framing over authenticated QUIC, validate the node public key against the registry, enforce request timeouts and concurrency limits, support byte ranges, and define redirect/error semantics before browser integration.
