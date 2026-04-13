# lume-servers-sidecar

Performs HTTP and TCP health checks on configured servers, tracks uptime history.

Produces `ServersPayload` payloads conforming to the VZGLYD sidecar channel ABI.

This sidecar is designed to be reusable. Any slide can depend on it via git and receive data payloads through the standard channel ABI.

## Poll Interval

Every 30 seconds.

## Payload Format

`ServersPayload` serialized as JSON bytes.

## Environment Variables

| Variable | Description |
|---|---|
| Config | Embedded `config/servers.json` defines server checks |

## Usage

Build the sidecar:

```bash
cargo build --target wasm32-wasip1 --release
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
