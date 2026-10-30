# greenhouse — CLAUDE.md

Monorepo: `backend/` (Rust workspace, see `backend/CLAUDE.md`) and `web/` (Angular SPA).

- Toolchains come from the root `flake.nix` via direnv; nothing is installed globally.
- Nix derivations live in `backend/nix/` and `web/nix/`; root and sub-flakes both call them.
- CI: `.github/workflows/backend-*.yml` and `web-*.yml`, path-filtered to their directory.
- Release tags are prefixed: `backend-v*`, `backend-example-v*`, `web-v*`.
- Task runner: root `justfile` with `backend` / `web` modules.
