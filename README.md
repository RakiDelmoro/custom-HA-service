# FlowPulse MQTT

Rust-based MQTT service converting pulse sensor data to water flow rates (L/min) for Home Assistant.

## Features

- **Gap Filling**: Fills time gaps with zeros
- **Duplicate Prevention**: Skips already-published timestamps
- **Auto-Reconnect**: Handles MQTT disconnections
- **Static Binary**: No runtime dependencies

## Architecture Support

`aarch64` | `amd64` | `armv7` | `armhf` | `i386`

## Quick Start

### 1. Build

```bash
# Install dependencies
rustup target add aarch64-unknown-linux-musl
cargo install cargo-zigbuild

# Download zig (one-time)
wget https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz
tar -xf zig-linux-x86_64-0.14.0.tar.xz
export PATH="$PWD/zig-linux-x86_64-0.14.0:$PATH"

# Build
cargo zigbuild --release --target aarch64-unknown-linux-musl
```

Binary: `target/aarch64-unknown-linux-musl/release/custom-ha-service`

### 2. Deploy to Home Assistant

**Via SCP:**
```bash
scp target/aarch64-unknown-linux-musl/release/custom-ha-service root@homeassistant.local:/usr/local/bin/
sudo chmod +x /usr/local/bin/custom-ha-service
```

**Via USB (offline):**
```bash
sudo cp /media/usb/custom-ha-service /usr/local/bin/
sudo chmod +x /usr/local/bin/custom-ha-service
```

### 3. Create Service

Create `/etc/systemd/system/custom-ha-service.service`:

```ini
[Unit]
Description=FlowPulse MQTT
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/custom-ha-service
Restart=always
Environment="MQTT_BROKER=homeassistant"
Environment="MQTT_PORT=1883"
Environment="MQTT_USERNAME="
Environment="MQTT_PASSWORD="
Environment="SUBSCRIBE_TOPIC=custom-service/in"
Environment="PUBLISH_TOPIC=flowpulse/out"
Environment="PULSES_PER_LITER=433"
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable custom-ha-service
sudo systemctl start custom-ha-service
journalctl -u custom-ha-service -f
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `MQTT_BROKER` | `homeassistant` | MQTT broker host/IP |
| `MQTT_PORT` | `1883` | Broker port |
| `MQTT_USERNAME` | - | Username |
| `MQTT_PASSWORD` | - | Password |
| `SUBSCRIBE_TOPIC` | `custom-service/in` | Input topic |
| `PUBLISH_TOPIC` | `flowpulse/out` | Output topic |
| `PULSES_PER_LITER` | `433` | Sensor pulses/liter |
| `RUST_LOG` | `info` | Log level |

## Data Format

**Input** → `custom-service/in`:
```json
{"total_pulses": 433, "Time_ms": 10000}
```

**Output** ← `flowpulse/out`:
```json
[
  {"timestamp": 1773658362, "flow_rate_lpm": 60.0},
  {"timestamp": 1773658363, "flow_rate_lpm": 60.0}
]
```

## Testing

```bash
mosquitto_pub -h homeassistant -t custom-service/in \
  -m '{"total_pulses": 433, "Time_ms": 10000}'

mosquitto_sub -h homeassistant -t flowpulse/out -v
```

## Development

```bash
export MQTT_BROKER=localhost
export MQTT_PORT=1883
export PULSES_PER_LITER=433
cargo run --release
```

## Troubleshooting

- **Permission denied**: `sudo chmod +x /usr/local/bin/custom-ha-service`
- **MQTT fails**: Check broker IP, port, credentials
- **No output**: Set `RUST_LOG=debug` in service file

## License

MIT
