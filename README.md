# `hither.host`

A local HTTPS proxy for a local HTTP server, which does not require self-signed
certificates.

## Installation

```sh
cargo install --git https://github.com/tjjfvi/hither.host
```

## Usage

Start a local http server listening on port 8080. Then run
```
hitherhost proxy localhost:8080 4433
```
and finally open https://hither.host:4433.

(Direct subdomains of `hither.host` (e.g. `foo.hither.host`) will also work.)

## How does this work?

The domain `hither.host` has an `A` record pointing to `127.0.0.1`, the loopback
address. So when your browser opens `hither.host:4433`, the request is handled
by the local server listening on port `4433`. The local hitherhost server uses a
public SSL certificate for `hither.host` (hosted on `cert.for.hither.host`), and
forwards all traffic to the local server.

## Why?

Getting a webapp to work over HTTPS can be a hassle, as there are many browser
restrictions that apply only to HTTPS connections. Hitherhost allows one to test
and troubleshoot these issues locally, without needing to set up self-signed
certs and the like.
