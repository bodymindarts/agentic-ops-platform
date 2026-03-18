# Agentic Ops Platform

## Dev Shell
All commands must run inside the nix dev shell:
```bash
nix develop -c <command>
```

## Standard Workflow
```bash
nix develop -c cargo fmt
nix develop -c cargo clippy --all-targets -- -D warnings
nix develop -c cargo nextest run
nix flake check
```

## Conventions
- Conventional commits: `type(scope): description`
- Binary name: `agentic-ops-server`
- Server port: 4200 (configurable via SERVER_PORT env var)
- GraphQL endpoint: POST /graphql
- GraphQL playground: GET /graphql

## Crate Structure
- `core` — domain types, config, error types (lib crate)
- `server` — axum HTTP server, GraphQL schema (bin crate)
