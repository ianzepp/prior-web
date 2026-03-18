# prior-web

Web client shell for Prior, intended to deploy separately from the `prior` daemon.

## Deploy Target

This repo is set up to deploy on Railway from GitHub using the root-level `Dockerfile`.

The app binds to Railway's injected `PORT` automatically. The server entrypoint also accepts:

- `PRIOR_WEB_SITE_ADDR` to override the bind address directly
- `PRIOR_GATE_WS_URL` to point `prior-web` at the Prior gate WebSocket listener
- `PRIOR_GATE_SERVICE_TOKEN` for service-authenticated gate access when enabled
- `PRIOR_WEB_GATE_ACTOR` to control the display name used for the current server-owned gate round trip

## Current State

This is still a skeleton Leptos app, but the integration boundary has moved to the server side. The browser now talks to `prior-web`, and `prior-web` performs a server-owned gate round trip over Prior's protobuf gate protocol via WebSocket.

The current implementation is a foundation step, not the final session model:

- the dashboard refresh is server-side
- `prior-web` opens a gate connection, performs `hello`, `door:connect`, `door:rooms`, and `door:disconnect`
- persistent per-user gate sessions and idle timeout policy are still planned work

## Local Dev Against Railway Gate

For local development, create a local `.env` from `.env.example`.

If you use `direnv`, this repo includes a root `.envrc` that loads `.env` automatically after:

```bash
direnv allow
```

Then run:

```bash
cargo leptos watch
```

The local server will use `PRIOR_GATE_WS_URL` and optional `PRIOR_GATE_SERVICE_TOKEN` from `.env` while still binding the web app locally.
