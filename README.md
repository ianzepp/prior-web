# prior-web

Web client shell for Prior, intended to deploy separately from the `prior` daemon.

## Deploy Target

This repo is set up to deploy on Railway from GitHub using the root-level `Dockerfile`.

The app binds to Railway's injected `PORT` automatically. The server entrypoint also accepts:

- `PRIOR_WEB_SITE_ADDR` to override the bind address directly
- `PRIOR_GATE_WS_URL` to point `prior-web` at the Prior gate WebSocket listener
- `PRIOR_GATE_SERVICE_TOKEN` for service-authenticated gate access when enabled
- `PRIOR_WEB_GATE_ACTOR` to control the display name used for the current server-owned gate round trip
- `GITHUB_*` and `PRIOR_WEB_SESSION_SECRET` to enable authenticated web sessions

## Current State

This is still a skeleton Leptos app, but the integration boundary has moved to the server side. The browser now talks to `prior-web`, and `prior-web` performs a server-owned gate round trip over Prior's protobuf gate protocol via WebSocket.

The current implementation is a foundation step, not the final session model:

- the dashboard refresh is server-side
- the public multi-user web trust boundary is `prior-web`
- GitHub OAuth login issues an app-owned secure session cookie
- `prior-web` stores reusable GitHub access-token state in the session so later repo actions can reuse it until expiry
- `prior-web` opens a gate connection, performs `hello`, `door:connect`, `door:rooms`, and `door:disconnect`
- authenticated gate calls are attributed with the GitHub-backed session user `sub`
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

The local server will use `PRIOR_GATE_WS_URL`, optional `PRIOR_GATE_SERVICE_TOKEN`, and the Auth0/session settings from `.env` while still binding the web app locally.
The local server will use `PRIOR_GATE_WS_URL`, optional `PRIOR_GATE_SERVICE_TOKEN`, and the GitHub/session settings from `.env` while still binding the web app locally.

## GitHub Auth Setup

To enable login locally or on Railway, configure:

- `GITHUB_CLIENT_ID`
- `GITHUB_CLIENT_SECRET`
- `GITHUB_CALLBACK_URL` or let it default to `PRIOR_WEB_BASE_URL + /auth/callback`
- `GITHUB_LOGOUT_RETURN_URL` or let it default to `PRIOR_WEB_BASE_URL + /`
- `PRIOR_WEB_BASE_URL`
- `PRIOR_WEB_SESSION_SECRET`

Optional:

- `GITHUB_OAUTH_SCOPES` to override the default GitHub OAuth scopes (`read:user user:email repo`)
- `PRIOR_WEB_SECURE_COOKIES=false` for local HTTP development

If GitHub OAuth is not configured, the app still starts, but authenticated routes and login are unavailable.
