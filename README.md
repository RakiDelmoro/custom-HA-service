# FlowPulse MQTT

Convert pulse sensor data to water flow rates (L/min) for Home Assistant.

## Overview

FlowPulse MQTT is a Rust-based service that listens for pulse sensor data via MQTT, calculates flow rates, and publishes the results back to MQTT. It supports automatic gap filling and duplicate detection.

- **Input**: Receives `{"total_pulses": 500, "Time_ms": 10000}` from MQTT
- **Calculates**: L/min = (pulses / seconds) × (60 / pulses_per_liter)
- **Timestamps**: Unix timestamps in seconds (UTC), one per second of data
- **Output**: Publishes JSON array with chronological entries
- **Gap Filling**: Automatically fills time gaps with zero entries

## Architecture Support

- `aarch64` (Home Assistant Green, Raspberry Pi 4, etc.)
- `amd64` (x86_64)
- `armv7`, `armhf` (Raspberry Pi 3, older ARM boards)
- `i386`

## Manual Installation

This guide covers building from source and manual installation on your system.

### Prerequisites

- **Rust** (stable) - Install via [rustup](https://rustup.rs/)
- **cargo** (included with Rust)
- Internet access for fetching crates

### Quick Build

#### For Home Assistant Green / ARM64 (aarch64)

The recommended method uses `cargo-zigbuild` for cross-compilation without requiring system cross-compilers:

```bash
# 1. Add Rust target
rustup target add aarch64-unknown-linux-musl

# 2. Install cargo-zigbuild
cargo install cargo-zigbuild

# 3. Install zig (one-time setup)
# Download from https://ziglang.org/download/
wget https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz
tar -xf zig-linux-x86_64-0.14.0.tar.xz
export PATH="$PWD/zig-linux-x86_64-0.14.0:$PATH"

# 4. Build static binary for aarch64
cargo zigbuild --release --target aarch64-unknown-linux-musl

# Binary location: target/aarch64-unknown-linux-musl/release/custom-ha-service
```

#### For Other Architectures

Replace `aarch64` with your target architecture:

```bash
# x86_64 (Intel/AMD)
rustup target add x86_64-unknown-linux-musl
cargo zigbuild --release --target x86_64-unknown-linux-musl

# ARMv7 (Raspberry Pi 3)
rustup target add armv7-unknown-linux-musleabihf
cargo zigbuild --release --target armv7-unknown-linux-musleabihf

# ARMhf (older Pi)
rustup target add arm-unknown-linux-musleabihf
cargo zigbuild --release --target arm-unknown-linux-musleabihf
```

### Verify the Binary

```bash
file target/aarch64-unknown-linux-musl/release/custom-ha-service
# Expected: ELF 64-bit LSB executable, ARM aarch64, version 1, statically linked, stripped
```

### Install on Your System

1. **Copy the binary** to your target machine:
   ```bash
   scp target/aarch64-unknown-linux-musl/release/custom-ha-service user@target:/usr/local/bin/
   # Or copy via USB
   ```

2. **Make it executable**:
   ```bash
   sudo chmod +x /usr/local/bin/custom-ha-service
   ```

3. **Create systemd service** file at `/etc/systemd/system/custom-ha-service.service`:

```ini
[Unit]
Description=FlowPulse MQTT Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/custom-ha-service
Restart=always
RestartSec=10

# MQTT Configuration - Customize these values
Environment="MQTT_BROKER=homeassistant"
Environment="MQTT_PORT=1883"
Environment="MQTT_USERNAME="
Environment="MQTT_PASSWORD="
Environment="SUBSCRIBE_TOPIC=custom-service/in"
Environment="PUBLISH_TOPIC=flowpulse/out"
Environment="CLIENT_ID=flowpulse-mqtt"
Environment="PULSES_PER_LITER=433"
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

4. **Enable and start the service**:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable custom-ha-service
   sudo systemctl start custom-ha-service
   ```

5. **Check status**:
   ```bash
   sudo systemctl status custom-ha-service
   journalctl -u custom-ha-service -f
   ```

## Configuration

All configuration is done via environment variables in the systemd service file:

| Variable | Default | Description |
|----------|---------|-------------|
| `MQTT_BROKER` | `homeassistant` | MQTT broker host or IP |
| `MQTT_PORT` | `1883` | MQTT broker port |
| `MQTT_USERNAME` | (empty) | MQTT username (if required) |
| `MQTT_PASSWORD` | (empty) | MQTT password (if required) |
| `SUBSCRIBE_TOPIC` | `custom-service/in` | Topic to receive sensor data |
| `PUBLISH_TOPIC` | `flowpulse/out` | Topic to publish flow rate data |
| `CLIENT_ID` | `flowpulse-mqtt` | MQTT client ID |
| `PULSES_PER_LITER` | `433` | Sensor pulses per liter |
| `RUST_LOG` | `info` | Log level: `debug`, `info`, `warn`, `error` |

### Edit Configuration

```bash
# Edit the service file
sudo nano /etc/systemd/system/custom-ha-service.service

# Reload and restart after changes
sudo systemctl daemon-reload
sudo systemctl restart custom-ha-service
```

## Data Format

### Input (MQTT Subscribe Topic)

Publish to your configured subscribe topic (default: `custom-service/in`):

```json
{
  "total_pulses": 433,
  "Time_ms": 10000
}
```

- `total_pulses`: Number of pulses detected by sensor
- `Time_ms`: Duration in milliseconds

### Output (MQTT Publish Topic)

The service publishes to your configured publish topic (default: `flowpulse/out`):

```json
[
  {"timestamp": 1773658362, "flow_rate_lpm": 60.0},
  {"timestamp": 1773658363, "flow_rate_lpm": 60.0},
  {"timestamp": 1773658364, "flow_rate_lpm": 60.0}
]
```

- `timestamp`: Unix epoch in seconds (UTC)
- `flow_rate_lpm`: Liters per minute

**Gap filling example** (when no data received):

```json
[
  {"timestamp": 1773658356, "flow_rate_lpm": 0.0},
  {"timestamp": 1773658357, "flow_rate_lpm": 0.0},
  {"timestamp": 1773658358, "flow_rate_lpm": 0.0},
  {"timestamp": 1773658359, "flow_rate_lpm": 60.0}
]
```

## Testing

Publish a test message:

```bash
mosquitto_pub -h homeassistant -p 1883 \
  -t custom-service/in \
  -m '{"total_pulses": 433, "Time_ms": 10000}'
```

Subscribe to output:

```bash
mosquitto_sub -h homeassistant -p 1883 \
  -t flowpulse/out -v
```

## Development

Build locally:

```bash
cargo build --release
```

Run locally with environment variables:

```bash
export MQTT_BROKER=localhost
export MQTT_PORT=1883
export SUBSCRIBE_TOPIC=custom-service/in
export PULSES_PER_LITER=433
./target/release/custom-ha-service
```

## Features

- **Gap Filling**: Automatically fills time gaps with zeros
- **Duplicate Prevention**: Skips already-published timestamps
- **Chronological Output**: Entries ordered oldest to newest
- **Event-Driven**: Only publishes when subscriber message arrives
- **Auto-Reconnect**: Handles MQTT disconnections gracefully
- **Static Binary**: No runtime dependencies

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Permission denied | `sudo chmod +x /usr/local/bin/custom-ha-service` |
| Binary wrong architecture | Rebuild with correct `--target` flag |
| MQTT connection fails | Check broker IP, port, credentials |
| No output | Add `Environment="RUST_LOG=debug"` and check logs |
| Build fails | Ensure `cargo-zigbuild` and `zig` are installed |

## License

MIT
