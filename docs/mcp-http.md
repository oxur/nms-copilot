# NMS Copilot HTTP MCP

NMS Copilot exposes MCP over RMCP Streamable HTTP when the REPL or headless
HTTP server is running.

## Endpoints

With the default local configuration:

- MCP endpoint: `http://127.0.0.1:5055/mcp`
- Health check: `http://127.0.0.1:5055/health`
- Discovery metadata: `http://127.0.0.1:5055/mcp-info`

The MCP endpoint is stateful. A client must initialize a session and include
the returned `Mcp-Session-Id` header on later requests.

## Lifecycle

1. `POST /mcp` with `initialize`, without `Mcp-Session-Id`.
2. Capture the `Mcp-Session-Id` response header.
3. `POST /mcp` with `notifications/initialized` and `Mcp-Session-Id`.
4. `POST /mcp` with `tools/list` or `tools/call` and `Mcp-Session-Id`.

## Required Headers

For `POST /mcp`:

```text
Content-Type: application/json
Accept: application/json, text/event-stream
```

For standalone `GET /mcp` SSE streams:

```text
Accept: text/event-stream
Mcp-Session-Id: <session id>
```

## Smoke Test

Use the built-in smoke command:

```sh
nms-copilot mcp-smoke --url http://127.0.0.1:5055/mcp
```

If `--url` is omitted, the command derives the URL from
`~/.nms-copilot/config.toml`:

```sh
nms-copilot mcp-smoke
```

For machine-readable output:

```sh
nms-copilot mcp-smoke --json
```

## Manual Curl Probe

Initialize:

```sh
curl -i \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  --data '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"probe","version":"1.0"}}}' \
  http://127.0.0.1:5055/mcp
```

Then send `notifications/initialized` with the returned `Mcp-Session-Id`:

```sh
curl -i \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'Mcp-Session-Id: <session id>' \
  --data '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  http://127.0.0.1:5055/mcp
```

List tools:

```sh
curl -i \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'Mcp-Session-Id: <session id>' \
  --data '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  http://127.0.0.1:5055/mcp
```
