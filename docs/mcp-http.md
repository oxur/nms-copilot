# NMS Copilot HTTP MCP

NMS Copilot exposes MCP over RMCP Streamable HTTP when the REPL or headless
HTTP server is running.

## Starting the server

A standard REPL build starts an HTTP observer over the same atlas used by the interactive session. For a server without the REPL:

```bash
nms-copilot --headless --http 127.0.0.1:5099 --save /path/to/save.hg
```

For an MCP client that launches a subprocess over stdio, omit `--http`:

```bash
nms-copilot --headless --save /path/to/save.hg
```

Stdio reserves stdout for protocol messages; diagnostics go to stderr. HTTP is included in the default build and can be omitted with the `http` feature disabled. Save watching and caching follow the application settings described in [configuration](configuration.md).

## Endpoints

With the default local configuration:

- MCP endpoint: `http://127.0.0.1:5099/mcp`
- Health check: `http://127.0.0.1:5099/health`
- Discovery metadata: `http://127.0.0.1:5099/mcp-info`

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

## Current location and resource subscriptions

Tools include `search_planets`, `plan_route`, `where_am_i`, `whats_nearby`, and system/base lookups. The server also exposes these model resources:

| URI | Contents |
|-----|----------|
| `nms://player/location` | Physical player location in its own galaxy |
| `nms://galaxy/stats` | Atlas statistics |
| `nms://bases` | Known player bases |

To follow location, use `resources/subscribe` for `nms://player/location`. An accepted application update invalidates the location resource through `notifications/resources/updated`; read it again to obtain current data. Notifications are invalidations, not snapshots or a replayable event history. Name/discovery changes also invalidate statistics, and base changes invalidate the bases resource.

The application applies live changes and attempts cache persistence before publishing observer notifications. It keeps the previous accepted state after invalid source data. See [live atlas behavior](live-atlas.md) for unknown locations, update boundaries, cache rebuilds, and library ownership.

## Smoke Test

Use the built-in smoke command:

```sh
nms-copilot mcp-smoke --url http://127.0.0.1:5099/mcp
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
  http://127.0.0.1:5099/mcp
```

Then send `notifications/initialized` with the returned `Mcp-Session-Id`:

```sh
curl -i \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'Mcp-Session-Id: <session id>' \
  --data '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  http://127.0.0.1:5099/mcp
```

List tools:

```sh
curl -i \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'Mcp-Session-Id: <session id>' \
  --data '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  http://127.0.0.1:5099/mcp
```
