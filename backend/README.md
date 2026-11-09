# greenhouse_backend

Rust workspace for the OpenGreenhouseManager backend: services, API gateways,
the shared DTOs (`greenhouse_core`) and the smart-device library
(`greenhouse_device`, host + ESP32). See `CLAUDE.md` for the layout and
`just --list` for the tasks.

- Services: `just start-all`, Docker images via `Dockerfile` (released on `v*` tags).
- Example devices: `greenhouse_device/examples/README.md`.
