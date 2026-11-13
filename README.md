# MiniWeb Server

MiniWeb is a small Rust HTTP server with a hand-written HTTP/1.x parser and response serializer. It has a shared protocol and handler layer, a blocking thread-pool mode, and a Tokio mode. It is a learning and portfolio project, not a claim of complete HTTP/1.1 compliance.

## Architecture

```text
TCP (threaded std / Tokio)
        |
 bounded connections + request timeouts
        |
 hand-written HTTP parser / response serializer
        |
 route dispatch -> API handlers / static file service
```

The parser, request/response types, router, handlers, and static file service do not depend on either I/O runtime. The server adapters own connection reads and writes.

## Supported HTTP subset

Supported methods are GET, POST, PUT, DELETE, HEAD, and OPTIONS. The parser accepts HTTP/1.0 and HTTP/1.1 request lines, case-insensitive headers, Content-Length bodies, and URL query percent decoding. Connections can send `Connection: close`; the default is persistent HTTP/1.1. Chunked request bodies receive 501. Malformed and unsupported-version requests receive 400 and 505. HEAD suppresses the body for handled static/API responses; OPTIONS returns a general Allow list.

This is not full HTTP/1.1: chunked transfer coding, request pipelining guarantees, `Expect: 100-continue`, ranges, conditional requests, upgrade, trailers, and full RFC validation are not implemented.  The async adapter currently processes one request per connection read cycle and does not yet guarantee keep-alive parsing of pipelined input.

## Build and run

Install Rust stable 1.85 or newer. Then:

```sh
cargo build
cargo run -- --mode=async
cargo run -- --mode=threaded
cargo test
cargo clippy --all-targets -- -D warnings
```

Set configuration with `configs/default.toml` and environment variables. `.env.example` documents variable names; MiniWeb does not load `.env` automatically. Environment variables override TOML. The default bind is `127.0.0.1:8080`.

## Routes

| Method | Path | Behavior |
|---|---|---|
| GET | `/` | `public/index.html` |
| GET | `/about` | `public/about.html` |
| GET | `/health` | Health JSON |
| GET | `/api/status` | Health JSON |
| GET | `/api/info` | Service metadata JSON |
| POST | `/api/data` | Validate and accept JSON |
| GET | other public paths | Static file, if safe and present |

```sh
curl http://127.0.0.1:8080/health
curl -X POST http://127.0.0.1:8080/api/data \
  -H 'Content-Type: application/json' -d '{"message":"hello"}'
```

## Configuration

| Key / environment variable | Default | Meaning |
|---|---:|---|
| `HOST` | `127.0.0.1` | Bind address |
| `PORT` | `8080` | TCP port |
| `MAX_CONNECTIONS` | `100` | Async connection permits / threaded queue bound |
| `REQUEST_TIMEOUT` | `30` | Read timeout in seconds |
| `STATIC_DIR` | `public` | Static file root |
| `MAX_HEADER_SIZE` | `16384` | Header byte limit |
| `MAX_BODY_SIZE` | `1048576` | Request body limit |
| `MAX_FILE_SIZE` | `10485760` | Static file limit |
| `WORKER_THREADS` | `4` | Threaded worker count |
| `LOG_LEVEL` | `info` | Tracing filter fallback |

`RUST_LOG` takes precedence through tracing-subscriber environment filtering. Binding to `0.0.0.0` exposes the server on all interfaces and emits a warning in async mode.

## Security notes

Static paths are percent-decoded, reject dot/parent components, reject dotfiles, canonicalized, and checked against the canonical public root (including symlink escapes). API JSON is validated before acceptance. Error bodies avoid filesystem paths and internal details. Keep limits appropriate for the deployment environment; this project has not undergone a security audit.

## Benchmarks

Criterion microbenchmarks are available with `cargo bench --bench server_benchmark`. On the development host (WSL2, x86_64; Rust 1.99.1; optimized bench profile), a run reported parser latency of **251.61–257.60 ns** and response serialization latency of **885.11–887.79 ns** (Criterion estimate intervals, 100 samples each). The router lookup benchmark measured **147.24–147.74 ns**. These are local microbenchmarks, not end-to-end server results. No load test comparing the two server modes has been run. For a comparison, build both modes in release on the same host and use the same endpoint, concurrency, request count, and client tool (for example `oha`); record host, command, and output before publishing numbers.

## Limitations and roadmap

The current implementation is an HTTP subset. Keep-alive and the std worker queue exist, but async connection parsing is still basic. The custom `Router` supports exact paths; the built-in server currently dispatches its fixed route table directly. Remaining work includes a shared router registration API in server setup, robust buffered multi-request parsing, graceful SIGINT/SIGTERM shutdown, precise total-header/body deadlines, complete method semantics, Date headers, broader adversarial tests, and measured load tests.
