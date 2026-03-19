# prior-web Auth Architecture And Integration Plan

## Context

`prior-web` is the public multi-user web surface for Prior. It is a separate
Leptos SSR service that talks to the Prior daemon over the gate WebSocket
protocol. The browser should never talk to the daemon directly.

This repo already follows the right broad integration shape:

- the browser talks to `prior-web`
- `prior-web` performs server-owned gate round trips
- gate access uses the protobuf `ClientHello`
- the daemon is a private backend service, not the public web boundary

The current gap is identity. Today, server-side gate requests are made with a
static server-owned actor name. That is a placeholder, not the intended user
session model.

This document adapts the architecture decisions from
`prior/docs/prior-web-incremental-conversion-plan.md` to this repo so auth and
later repo access work can be tracked locally against `prior-web`.

## Decisions

- **Public trust boundary**: `prior-web`, not the daemon, is the public
  multi-user trust boundary in v1.
- **Login identity**: Auth0 is the web login identity provider.
- **Repo authorization**: GitHub App installations are the repo access model.
  This is separate from login identity.
- **Daemon trust model**: the daemon trusts `prior-web` via gate
  `service_token`, not via user OAuth tokens.
- **User identity propagation**: `prior-web` sends the authenticated Auth0
  `sub` claim as `ClientHello.user_id` when opening gate connections.
- **Session model**: `prior-web` owns a normal secure web session cookie and
  derives current user identity from that session on the server.
- **Credential boundary**: the daemon never stores GitHub credentials. Any
  GitHub-facing action later receives fresh credentials only for that explicit
  operation.
- **v1 enforcement point**: repo access is enforced in `prior-web`, not in the
  daemon kernel.

## What This Means For This Repo

### Current State

The existing implementation in this repo already establishes the key server-side
integration seam:

- [`src/net/prior_gate.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/net/prior_gate.rs)
  performs the gate round trip from the server
- [`proto/prior/gate.proto`](/Users/ianzepp/github/ianzepp/prior-web/proto/prior/gate.proto)
  already includes `ClientHello.user_id`
- [`src/runtime.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/runtime.rs)
  currently provides a static `PRIOR_WEB_GATE_ACTOR`

That means auth does not require a transport redesign. The main architectural
change is to replace the placeholder actor with a real authenticated user
identity sourced from a web session.

### Intended State

The intended request path is:

```text
Browser -> prior-web -> Auth0 session cookie -> server route/server function
        -> gate connection with { service_token, user_id = auth0_sub }
        -> Prior daemon
```

The browser should not hold Prior credentials or talk to Prior directly.

## Auth Architecture

### 1. Axum Owns OIDC And Session Issuance

OIDC mechanics should live in the Axum host layer, not in client-side Leptos
code.

`prior-web` should expose:

- `GET /auth/login`
- `GET /auth/callback`
- `POST /auth/logout` or `GET /auth/logout`

Responsibilities:

- redirect to Auth0 authorize endpoint
- validate callback `state`
- exchange authorization code for tokens
- validate identity claims
- issue a secure application session cookie
- clear that cookie on logout

Leptos should consume authenticated user state, not implement the OIDC flow.

### 2. prior-web Owns The Session Cookie

Use a secure, HTTP-only, signed or encrypted cookie to represent the app
session.

The session should carry only app-owned session data such as:

- `user_sub`
- `display_name`
- `email`
- `avatar_url`
- `auth_time`
- `expires_at`

Do not make raw Auth0 or GitHub tokens the foundation of normal page rendering
or server function authorization in v1.

### 3. Current User Is A First-Class Server Concept

Each incoming request should resolve to either:

- `Anonymous`
- `Authenticated(CurrentUser)`

`CurrentUser` should be the canonical identity object used by:

- Axum route handlers
- Leptos SSR context setup
- server functions
- gate client calls

Suggested fields:

```rust
struct CurrentUser {
    sub: String,
    display_name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}
```

The `sub` field is the identity that should cross the gate boundary as
`ClientHello.user_id`.

## UI Architecture

### App Root

The app root should provide one auth context into Leptos during SSR:

- `AuthState::Anonymous`
- `AuthState::Authenticated(CurrentUser)`

This belongs in the application composition root, likely through
[`src/app.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/app.rs), so pages do
not discover auth independently.

### Route Shape

Recommended route split:

- `/`
  - public landing/login screen
- `/app`
  - authenticated application shell

Unauthenticated users should not execute server-owned dashboard work just to be
redirected after the fact.

### First UI Milestone

The first auth UI pass should be intentionally narrow:

Unauthenticated:

- product framing
- login call to action
- no gate refresh

Authenticated:

- user identity visible in shell
- logout action
- dashboard refresh allowed

This keeps the trust boundary clean before repo workflows exist.

## Gate Integration

### Gate Hello Contract

The gate `ClientHello` already has the fields this repo needs:

- `service_token`
- `user_id`

`prior-web` should set them as follows:

- `service_token`: shared secret proving the caller is trusted `prior-web`
- `user_id`: Auth0 `sub` for the current authenticated session

That means the current static server actor should stop being the user identity
source for authenticated requests.

### refresh_dashboard Policy

[`refresh_dashboard`](/Users/ianzepp/github/ianzepp/prior-web/src/net/prior_gate.rs)
should become the reference policy for secure server functions:

- resolve `CurrentUser`
- reject unauthenticated requests
- call gate using `CurrentUser.sub` as `ClientHello.user_id`
- report authenticated status in returned UI state

This is the first real boundary, not just a login page.

### v1 Access Model

The daemon does not enforce GitHub repo ACLs in v1.

Instead:

- `prior-web` checks what repos the user can access
- `prior-web` only sends room and repo operations the user is allowed to perform
- the daemon trusts those requests because they came from authenticated
  `prior-web` over the gate service boundary

This is acceptable only if the daemon is not exposed as a public general-purpose
multi-user endpoint.

## GitHub Relationship

Auth0 and GitHub solve different problems.

Auth0 answers:

- who is this human?

GitHub App installation access answers:

- what repos can this human operate on?

Even if Auth0 uses GitHub as a social login connection, that does not replace
the separate GitHub App repo-access model described in the sibling project plan.

For this repo, that means:

- implement Auth0-backed user sessions first
- add GitHub App repo flows later
- do not conflate login identity with repo authorization

## Module Plan For prior-web

Recommended additions:

```text
src/
  auth/
    mod.rs
    config.rs
    routes.rs
    session.rs
    user.rs
  state/
    auth.rs
```

Responsibilities:

- `config.rs`
  - env parsing and validation for Auth0 and cookie settings
- `routes.rs`
  - `/auth/login`, `/auth/callback`, `/auth/logout`
- `session.rs`
  - session cookie encode/decode and expiration handling
- `user.rs`
  - `CurrentUser` extraction for request and server-function use
- `state/auth.rs`
  - Leptos-facing auth state and shared types

This keeps auth separate from:

- gate transport code
- dashboard page logic
- runtime env unrelated to identity

## Runtime Configuration

`prior-web` should eventually need configuration in this general shape:

- `PRIOR_GATE_WS_URL`
- `PRIOR_GATE_SERVICE_TOKEN`
- `AUTH0_DOMAIN`
- `AUTH0_CLIENT_ID`
- `AUTH0_CLIENT_SECRET`
- `AUTH0_CALLBACK_URL`
- `AUTH0_LOGOUT_RETURN_URL`
- `PRIOR_WEB_BASE_URL`
- `PRIOR_WEB_SESSION_SECRET`

Notes:

- `PRIOR_WEB_GATE_ACTOR` is acceptable as a temporary local-dev fallback for
  non-user service operations, but it should not remain the source of user
  identity once auth lands.
- `prior-web` should remain stateless aside from secure cookies; it does not
  need persistent disk for this auth phase.

## Security Constraints

- the browser must not talk directly to the daemon
- the browser must not hold Prior-specific credentials
- GitHub credentials must not be persisted in daemon state
- secrets must not be logged, traced, or written into durable state
- unauthenticated requests must fail explicitly, not silently downgrade to a
  shared actor identity

## Rollout Plan

### Phase 1: Auth Foundation

- add Auth0 config parsing
- add Axum auth routes
- add secure session cookie issuance
- add `CurrentUser` extraction

### Phase 2: SSR Auth Context

- provide `AuthState` at the app root
- add public landing/login view
- add authenticated app shell

### Phase 3: Secure Gate Usage

- require authenticated session for dashboard server functions
- replace static actor identity with `CurrentUser.sub`
- surface authenticated identity in the UI

### Phase 4: Repo Access Work

- add GitHub App installation-based repo discovery
- enforce repo access in `prior-web`
- use explicit provider-facing actions for import/sync/publish

## Open Questions

- whether Auth0 should use GitHub social login immediately or whether another
  Auth0 connection should back initial login
- whether logout should be local-session-only or also redirect through Auth0
  upstream logout
- whether session cookies should be encrypted or only signed in the first pass
- whether server functions should consume `CurrentUser` through request context,
  a helper extractor, or a wrapper utility

These do not change the core architecture. The key invariant is that
`prior-web`, not the browser and not the daemon, owns the authenticated
multi-user web session boundary.
