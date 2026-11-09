# greenhouse

Monorepo of the Open Greenhouse Manager.

| Directory  | What                                         |
| ---------- | -------------------------------------------- |
| `backend/` | Rust workspace: services, APIs, core library |
| `web/`     | Angular frontend (SPA)                       |

## Development

`direnv allow` loads the root Nix devShell (Rust stable from `backend/rust-toolchain.toml` + Node 24). Per-project
shells: `nix develop .#backend` / `nix develop .#web`.

```sh
just                 # list recipes
just backend <recipe>
just web <recipe>
just lint            # both projects
just test
```

## Builds & releases

- `nix build .#backend` / `nix build .#web`
- Sub-flakes stay usable standalone: `github:OpenGreenhouseManager/greenhouse?dir=backend`
- `vX.Y.Z` releases backend and web together
  (older releases use the per-project `backend-v*` / `web-v*` tags).
