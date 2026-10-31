# greenhouse

Monorepo of the Open Greenhouse Manager.

| Directory  | What                                         |
| ---------- | -------------------------------------------- |
| `backend/` | Rust workspace: services, APIs, core library |
| `web/`     | Angular frontend (SPA)                       |

## Development

`direnv allow` loads the root Nix devShell (Rust nightly + Node 24). Per-project
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
- `vX.Y.Z` releases backend and web together; `backend-example-vX.Y.Z` releases the example images
  (older releases use the per-project `backend-v*` / `web-v*` tags).
