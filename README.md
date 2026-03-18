# prior-web

Web client shell for Prior, intended to deploy separately from the `prior` daemon.

## Deploy Target

This repo is set up to deploy on Railway from GitHub using the root-level `Dockerfile`.

The app binds to Railway's injected `PORT` automatically. The server entrypoint also accepts:

- `PRIOR_WEB_SITE_ADDR` to override the bind address directly
- `PRIOR_GATE_WS_URL` to point the browser client at a Prior gate websocket

If `PRIOR_GATE_WS_URL` is unset, the frontend falls back to `ws://<current-host>:7071/`.

## Current State

This is still a skeleton Leptos app. It is not yet protocol-compatible with the sibling `prior` gate service, which currently expects binary protobuf websocket envelopes rather than the placeholder text frames sent by this repo.
