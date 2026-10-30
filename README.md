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
- Tags are prefixed per project: `backend-vX.Y.Z`, `backend-example-vX.Y.Z`, `web-vX.Y.Z`
  (pre-merge tags were renamed the same way).
