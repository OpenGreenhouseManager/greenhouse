# greenhouse_device examples

Example smart devices built with `greenhouse_device`. The host examples and the
ESP32 firmware use the same API (`DeviceBuilder`, handlers taking
`Arc<Config<T>>`, `trigger_alert`); only `main` differs.

| Example | Runs on | Kind | Port | What it does |
|---|---|---|---|---|
| `input_output_int_saver` | host | hybrid | 6001 | Stores a number; `/write` answers 500 outside `min..=max` |
| `input_alert_trigger` | host | input | 6002 | Sends `low_alert` / `high_alert` when a written number is outside `min..=max` |
| `periodic_alert` | host | output | 6003 | Sends a random alert every `interval` (+ `random_jitter`) seconds; `/read` returns the counts |
| `esp32_input_output_int_saver` | ESP32-C3 | hybrid | 6001 | `input_output_int_saver` as firmware, plus an out-of-range alert |

Every device serves `GET /read`, `POST /write`, `GET /status`, `GET`/`POST /config`
and `POST /activate`. A route its kind has no handler for answers 404.

## Host

```fish
# all three (from backend/)
just device
just device-stop

# one, with an optional config path (default: ./config/<example>/config.json)
cargo run -p greenhouse_device --example input_output_int_saver -- /path/to/config.json
```

The config file is created with the example's default config on first start
and rewritten on `POST /config` and `POST /activate`:

```json
{
  "port": 6001,
  "datasource_id": "7a224a14-6e07-45a3-91da-b7584a5731c1",
  "additional_config": { "min": 0, "max": 100 },
  "scripting_api": null
}
```

## ESP32-C3

Wi-Fi credentials are compiled in. With the board on USB (from `backend/`):

```fish
WIFI_SSID=my-net WIFI_PASSWORD=secret just esp32-flash
# or: cargo esp32-run --example esp32_input_output_int_saver
```

The config lives in the `nvs` flash partition; the default config in `main` is
used until the device has stored one. On a host, the ESP32 examples build to a
stub `main`.

## Writing your own device

Copy the example closest to your device. Define your `additional_config` type,
then implement the handlers. Name firmware examples `esp32_*.rs` so the ESP32
lint (`just esp32-lint`) picks them up and the host-only ones are skipped.
