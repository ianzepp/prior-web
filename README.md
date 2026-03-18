# prior-web

Web client shell for Prior, intended to deploy separately from the `prior` daemon.

## Deploy Target

This repo is set up to deploy on Railway from GitHub using the root-level `Dockerfile`.

The app binds to Railway's injected `PORT` automatically. The server entrypoint also accepts:

- `PRIOR_WEB_SITE_ADDR` to override the bind address directly
- `PRIOR_GATE_TCP_ADDR` to point `prior-web` at the Prior gate TCP listener
- `PRIOR_GATE_SERVICE_TOKEN` for service-authenticated gate access when enabled
- `PRIOR_WEB_GATE_ACTOR` to control the display name used for the current server-owned gate round trip

## Current State

This is still a skeleton Leptos app, but the integration boundary has moved to the server side. The browser now talks to `prior-web`, and `prior-web` performs a server-owned gate round trip over Prior's protobuf TCP protocol.

The current implementation is a foundation step, not the final session model:

- the dashboard refresh is server-side
- `prior-web` opens a gate connection, performs `hello`, `door:connect`, `door:rooms`, and `door:disconnect`
- persistent per-user gate sessions and idle timeout policy are still planned work
