# prior-web Auth Implementation Plan

## Goal

Turn `prior-web` from a server-owned demo shell with a static gate actor into an
authenticated web surface where:

- Auth0 establishes user identity
- `prior-web` owns the secure web session
- server functions run in the context of the authenticated user
- gate connections send `ClientHello.user_id = auth0_sub`
- unauthenticated users cannot execute Prior-facing server work

This plan is intentionally scoped to login identity and gate integration. It
does not yet include GitHub App repo access work.

## Current Repo Baseline

### Existing Assets

- [`src/main.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/main.rs)
  mounts Leptos SSR routes and static assets on Axum.
- [`src/app.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/app.rs)
  provides the UI shell and routes `/` and `/app` to the same dashboard page.
- [`src/net/prior_gate.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/net/prior_gate.rs)
  runs the current server-owned gate refresh round trip.
- [`src/runtime.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/runtime.rs)
  reads gate configuration and provides the temporary static actor identity.
- [`proto/prior/gate.proto`](/Users/ianzepp/github/ianzepp/prior-web/proto/prior/gate.proto)
  already supports `ClientHello.user_id`.

### Architectural Gap

The current code already has the right transport boundary, but not the right
identity source. The main change is:

```text
before: gate user_id = static env actor
after:  gate user_id = authenticated Auth0 sub from prior-web session
```

## Workstreams

## 1. Auth Server Foundation

### Outcome

Axum owns login, callback, logout, and session issuance.

### New Modules

Create:

```text
src/auth/
  mod.rs
  config.rs
  routes.rs
  session.rs
  user.rs
```

### Responsibilities

- `config.rs`
  - parse and validate Auth0 and session env vars
  - expose one `AuthConfig` type
- `routes.rs`
  - implement `/auth/login`
  - implement `/auth/callback`
  - implement `/auth/logout`
- `session.rs`
  - encode/decode session cookie payload
  - manage cookie attributes and expiration
- `user.rs`
  - define `CurrentUser`
  - provide extraction helpers for request handlers and server code

### Notes

- OIDC should be handled in Axum, not in browser-side Leptos code.
- Use a secure, HTTP-only cookie.
- Treat missing or invalid session state as anonymous, not as a fallback shared
  user.

### Acceptance Criteria

- `/auth/login` redirects to Auth0 with state protection
- `/auth/callback` validates the response and sets a session cookie
- `/auth/logout` clears the session cookie
- authenticated requests can resolve a `CurrentUser`

## 2. App State And Route Split

### Outcome

Leptos receives authenticated user state from SSR context and the route tree
stops treating `/` and `/app` as the same anonymous dashboard.

### Files To Change

- [`src/app.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/app.rs)
- add [`src/state/auth.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/state/auth.rs)
- update [`src/state/mod.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/state/mod.rs)
- likely add one landing page module under
  [`src/pages/`](/Users/ianzepp/github/ianzepp/prior-web/src/pages)

### Responsibilities

- define `AuthState`
- provide auth state at the app root
- route `/` to a public landing/login page
- route `/app` to an authenticated shell

### UI Rules

Unauthenticated:

- show login CTA
- do not trigger dashboard gate refresh

Authenticated:

- show user identity in shell
- show logout affordance
- allow dashboard interactions

### Acceptance Criteria

- the app root provides a stable auth context
- `/` renders correctly without session state
- `/app` behaves as an authenticated area, not a second copy of the public page

## 3. Axum Integration And Request Context

### Outcome

The Axum host composes auth routes, Leptos SSR routes, static assets, and
request-scoped auth resolution cleanly.

### Files To Change

- [`src/main.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/main.rs)

### Responsibilities

- mount auth routes before Leptos routes
- initialize any shared auth config/state needed by handlers
- make current-user resolution available to SSR and server functions

### Design Constraint

Do not bury auth inside ad hoc helpers attached only to one handler. The Axum
composition root should make the trust boundary obvious.

### Acceptance Criteria

- auth routes and SSR routes coexist cleanly
- route composition makes it obvious where web identity enters the app

## 4. Server Function Authorization

### Outcome

Prior-facing server functions require an authenticated web session.

### Files To Change

- [`src/net/prior_gate.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/net/prior_gate.rs)

### Responsibilities

- resolve `CurrentUser` in `refresh_dashboard`
- reject unauthenticated access explicitly
- pass `CurrentUser.sub` into gate `ClientHello.user_id`
- remove reliance on `PRIOR_WEB_GATE_ACTOR` for authenticated requests

### Behavioral Shift

The dashboard server refresh becomes the reference implementation for all future
Prior-facing server actions:

- no session -> no gate work
- valid session -> gate work attributed to Auth0 `sub`

### Acceptance Criteria

- anonymous calls to `refresh_dashboard` fail cleanly
- authenticated calls set gate `user_id` from the session identity
- returned UI state reflects the real authenticated user context

## 5. Runtime Configuration Cleanup

### Outcome

Runtime config reflects the real auth boundary instead of only the temporary
demo gate actor setup.

### Files To Change

- [`src/runtime.rs`](/Users/ianzepp/github/ianzepp/prior-web/src/runtime.rs)
- [`.env.example`](/Users/ianzepp/github/ianzepp/prior-web/.env.example)
- [`README.md`](/Users/ianzepp/github/ianzepp/prior-web/README.md)

### Responsibilities

- add typed Auth0 config parsing
- add session-secret and base-URL settings
- document local-dev configuration
- demote `PRIOR_WEB_GATE_ACTOR` to optional fallback-only status or remove it if
  no longer justified

### Acceptance Criteria

- env requirements are documented
- runtime config has one clear place for auth settings
- docs describe the trust boundary accurately

## Proposed Execution Order

1. Add auth config and shared auth types.
2. Add Axum auth routes and secure session handling.
3. Thread current-user resolution into SSR setup.
4. Split public and authenticated UI routes.
5. Protect `refresh_dashboard` and switch gate identity to Auth0 `sub`.
6. Update `.env.example` and README.

This order keeps identity and trust-boundary work ahead of visual polish.

## Suggested Type Shapes

These are not hard requirements, but they are the intended shape for v1.

```rust
struct AuthConfig {
    domain: String,
    client_id: String,
    client_secret: String,
    callback_url: String,
    logout_return_url: String,
    base_url: String,
    session_secret: String,
}

struct CurrentUser {
    sub: String,
    display_name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

enum AuthState {
    Anonymous,
    Authenticated(CurrentUser),
}
```

## Route Contract

### `/auth/login`

- input: optional return destination
- output: redirect to Auth0 authorize endpoint

### `/auth/callback`

- input: authorization code + state
- output: validated session cookie + redirect to `/app`

### `/auth/logout`

- input: current session cookie
- output: cleared session cookie + redirect to public entry

## Security Assertions

- unauthenticated requests must not silently use a shared gate actor
- the browser must never call Prior directly
- the browser must never be treated as the trust boundary
- the daemon must not become the place where end-user web auth is interpreted
- Auth0 identity and GitHub repo authorization must remain separate concerns

## Explicit Non-Goals In This Plan

- GitHub App installation UI
- repo listing/import/sync/publish UX
- daemon-side ACL enforcement
- long-lived per-user gate session persistence
- organization/team tenancy features

Those are downstream work. This plan only establishes the correct web auth
boundary and identity propagation model.

## Done Definition

This plan is complete when all of the following are true:

- a user can log in through Auth0
- `prior-web` stores an app-owned secure session cookie
- the app renders different public and authenticated entry states
- `refresh_dashboard` requires authentication
- gate `ClientHello.user_id` comes from the authenticated Auth0 `sub`
- the repo docs and env configuration describe this model accurately
