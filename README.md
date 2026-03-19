# FlowPulse MQTT

Rust-based MQTT service converting pulse sensor data to water flow rates (L/min) for Home Assistant.

## Features

- **Gap Filling**: Fills time gaps with zeros
- **Duplicate Prevention**: Skips already-published timestamps
- **Auto-Reconnect**: Handles MQTT disconnections
- **Static Binary**: No runtime dependencies

## Architecture Support

`aarch64` (ARM64) - Tested on Home Assistant Green and Raspberry Pi 4

## Installation via USB (Offline)

This method is perfect for Home Assistant devices without internet access or when you prefer offline installation.

### Prerequisites

- **Build Computer**: Linux/macOS with Rust installed
- **USB Drive**: Any USB stick (FAT32/EXT4/NTFS supported)
- **Home Assistant Device**: aarch64 architecture with terminal access (SSH or direct)

---

### Step 1: Build the Binary (On Your Build Computer)

Open a terminal on your development/build computer:

```bash
# Install required Rust target
rustup target add aarch64-unknown-linux-musl

# Install cargo-zigbuild (no sudo needed)
cargo install cargo-zigbuild

# Download and setup zig (one-time setup)
wget https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz
tar -xf zig-linux-x86_64-0.14.0.tar.xz
export PATH="$PWD/zig-linux-x86_64-0.14.0:$PATH"

# Build static binary for aarch64
cargo zigbuild --release --target aarch64-unknown-linux-musl
```

**Output location:** `target/aarch64-unknown-linux-musl/release/custom-ha-service`

---

### Step 2: Copy Binary to USB Drive (On Your Build Computer)

Plug your USB drive into the build computer, then:

**Via terminal:**
```bash
# Find your USB mount point (examples: /media/username/USB_NAME or /mnt/usb)
# Copy the binary to USB
cp target/aarch64-unknown-linux-musl/release/custom-ha-service /media/YOUR_USERNAME/YOUR_USB_NAME/
```

**Via file manager:** Simply drag the `custom-ha-service` file from `target/aarch64-unknown-linux-musl/release/` to your USB drive.

**Safely eject USB** from your build computer.

---

### Step 3: Transfer to Home Assistant (On HA Device)

1. **Plug USB into your Home Assistant device**

2. **Access HA terminal** via one of these methods:
   - SSH: `ssh root@homeassistant.local`
   - Terminal & SSH addon in Home Assistant UI
   - Direct keyboard/monitor connection

3. **Copy binary from USB to system:**

```bash
# Check USB mount location
ls /media

# Copy to system directory
sudo cp /media/YOUR_USB_NAME/custom-ha-service /usr/local/bin/

# Make it executable
sudo chmod +x /usr/local/bin/custom-ha-service

# Verify it works
/usr/local/bin/custom-ha-service --help
```

---

### Step 4: Create Systemd Service (On HA Device)

Create the service file:

```bash
sudo tee /etc/systemd/system/custom-ha-service.service << 'EOF'
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
EOF
```

**Start the service:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable custom-ha-service
sudo systemctl start custom-ha-service
```

**Check status:**

```bash
sudo systemctl status custom-ha-service
journalctl -u custom-ha-service -f
```

---

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

| Issue | Solution |
|-------|----------|
| **USB not showing up** | Check `/media` and `/mnt` directories |
| **Permission denied** | Run `sudo chmod +x /usr/local/bin/custom-ha-service` |
| **Binary won't run** | Verify architecture: `file /usr/local/bin/custom-ha-service` should show "ARM aarch64" |
| **MQTT connection fails** | Check broker IP, port, credentials |
| **No output** | Set `RUST_LOG=debug` in service file |

## License

MIT
