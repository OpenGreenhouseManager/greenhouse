# greenhouse — CLAUDE.md

Monorepo: `backend/` (Rust workspace, see `backend/CLAUDE.md`) and `web/` (Angular SPA).

- Toolchains come from the root `flake.nix` via direnv; nothing is installed globally.
- Nix derivations live in `backend/nix/` and `web/nix/`; root and sub-flakes both call them.
- CI: `.github/workflows/ci.yml` is the gate; it calls `backend-ci.yml` / `web-ci.yml` based on changed paths. `CI OK` is the only required check on `main`. Release workflows (`backend-docker`, `backend-examples`, `web-release`) run on tags.
- Release tags are prefixed: `backend-v*`, `backend-example-v*`, `web-v*`.
- Task runner: root `justfile` with `backend` / `web` modules.
