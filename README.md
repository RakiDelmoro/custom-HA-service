# Water Flow MQTT Service

MQTT service that converts pulse sensor data to flow rates (L/min) for Home Assistant.

## What It Does

- **Input**: Receives `{"total_pulses": 500, "Time_ms": 10000}` from MQTT
- **Calculates**: L/min = (pulses / seconds) × (60 / pulses_per_liter)
- **Timestamps**: Generates timestamps (UTC+8) for each second of data
- **Output**: Publishes JSON array with chronological entries
- **Gap Filling**: Automatically fills time gaps with zero entries (no data missed)
- **Publishes**: Only when subscriber message arrives (no periodic output)

## Quick Install

Add to Home Assistant:

1. Go to **Settings** → **Add-ons** → **Add-on Store**
2. Click **⋮** → **Repositories** → Add `https://github.com/YOUR_USERNAME/custom-ha-service`
3. Find "Custom HA Service" → **Install** → **Start**

## Configuration

| Option | Default | Description |
|--------|---------|-------------|
| `mqtt_broker` | `homeassistant` | MQTT broker host |
| `mqtt_port` | `1883` | MQTT port |
| `mqtt_username` | `""` | MQTT username (optional) |
| `mqtt_password` | `""` | MQTT password (optional) |
| `subscribe_topic` | `custom-service/in` | Input topic |
| `publish_topic` | `custom-service/out` | Output topic |
| `client_id` | `custom-ha-service` | MQTT client ID |
| `pulses_per_liter` | `433` | Your sensor's pulses per liter |

## Integration Example

**Send to `custom-service/in`:**
```bash
mosquitto_pub -t custom-service/in -m '{"total_pulses": 433, "Time_ms": 10000}'
```

**Receive on `custom-service/out`:**
```json
[
  {"timestamp": "25 Feb 2026 10:00:20", "flow_rate_lpm": 60.0},
  {"timestamp": "25 Feb 2026 10:00:21", "flow_rate_lpm": 60.0},
  {"timestamp": "25 Feb 2026 10:00:22", "flow_rate_lpm": 60.0}
]
```

**Gap Example (5s gap detected):**
```json
[
  {"timestamp": "25 Feb 2026 10:00:16", "flow_rate_lpm": 0.0},
  {"timestamp": "25 Feb 2026 10:00:17", "flow_rate_lpm": 0.0},
  {"timestamp": "25 Feb 2026 10:00:18", "flow_rate_lpm": 0.0},
  {"timestamp": "25 Feb 2026 10:00:19", "flow_rate_lpm": 60.0},
  {"timestamp": "25 Feb 2026 10:00:20", "flow_rate_lpm": 60.0}
]
```

## Input Format

```json
{
  "total_pulses": 433,
  "Time_ms": 10000
}
```

- `total_pulses`: Number of pulses detected by sensor
- `Time_ms`: Duration in milliseconds over which pulses were counted

## Output Format

Always returns JSON array, even for single entry:

```json
[
  {
    "timestamp": "25 Feb 2026 10:00:20",
    "flow_rate_lpm": 60.0
  }
]
```

- `timestamp`: Format "DD MMM YYYY HH:MM:SS" (UTC+8)
- `flow_rate_lpm`: Liters per minute (0.0 for gaps)

## Features

- **Always Array**: Single or multiple entries, always JSON array
- **Gap Filling**: Automatically fills missing seconds with zeros
- **Duplicate Prevention**: Skips already-published timestamps
- **Chronological**: Entries ordered oldest to newest
- **Silent Between Messages**: No output when no data received
- **Auto-Reconnect**: Handles MQTT disconnections gracefully

## Architecture

Supports: `aarch64`, `amd64`, `armv7`, `armhf`, `i386`

## Development

```bash
# Build
cargo build --release

# Run locally
export MQTT_BROKER=localhost
export MQTT_PORT=1883
export SUBSCRIBE_TOPIC=custom-service/in
export PUBLISH_TOPIC=custom-service/out
export PULSES_PER_LITER=433
./target/release/custom-ha-service
```

## License

MIT
