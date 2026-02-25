# Custom Home Assistant Service

A custom MQTT service for Home Assistant Green that subscribes to MQTT topics, processes messages, and publishes responses. This service runs continuously as long as your Home Assistant Green is powered on.

## Features

- **MQTT Subscribe/Publish**: Subscribes to incoming MQTT messages and publishes processed responses
- **Automatic Reconnection**: Handles connection failures and automatically reconnects to the MQTT broker
- **Cross-platform**: Supports all Home Assistant architectures (aarch64, amd64, armhf, armv7, i386)
- **Persistent Session**: Maintains MQTT session state across reconnections
- **Configurable**: All settings configurable through Home Assistant UI

## Architecture Support

This addon is designed for Home Assistant Green and supports:
- **aarch64** (ARM64) - Primary target for HA Green
- amd64 (x86_64)
- armhf
- armv7
- i386

## Installation

### Method 1: Add Custom Repository (Recommended)

The easiest way to install this add-on is by adding this repository to your Home Assistant:

1. **Copy the repository URL:**
   ```
   https://github.com/yourusername/custom-ha-service
   ```
   *Replace `yourusername` with your actual GitHub username*

2. **In Home Assistant:** Go to **Settings** → **Add-ons** → **Add-on Store**

3. **Add Repository:** Click the **⋮** menu (top right) → **Repositories**

4. **Paste URL:** Enter the repository URL and click **Add**

5. **Install:** Find "Custom HA Service" in the store, click on it, then click **Install**

6. **Configure:** Set your MQTT settings (see Configuration section below)

7. **Start:** Click **Start** to run the service

### Method 2: Local Addon Repository

For local installation without GitHub:

1. Copy the `custom-ha-service` folder to your Home Assistant addons directory:
    ```bash
    # On Home Assistant OS/Container
    mkdir -p /addons/custom-ha-service
    cp -r custom-ha-service/* /addons/custom-ha-service/
    ```

2. In Home Assistant, go to **Settings** → **Add-ons** → **Add-on Store**

3. Click the **⋮** menu → **Check for updates**

4. You should see "Custom HA Service" in the "Local Add-ons" section

5. Click on it and then click **Install**

6. Configure the options (see Configuration section below)

7. Click **Start**

### Method 3: Development/Testing

To build and test locally:

```bash
# Build for your current architecture
cargo build --release

# Run with environment variables
export MQTT_BROKER=homeassistant
export MQTT_PORT=1883
export SUBSCRIBE_TOPIC=custom-service/in
export PUBLISH_TOPIC=custom-service/out
export CLIENT_ID=custom-ha-service
./target/release/custom-ha-service
```

## Configuration

Configure through Home Assistant UI or edit `options` in `config.yaml`:

| Option | Default | Description |
|--------|---------|-------------|
| `mqtt_broker` | `homeassistant` | MQTT broker hostname/IP |
| `mqtt_port` | `1883` | MQTT broker port |
| `mqtt_username` | (empty) | MQTT username (optional) |
| `mqtt_password` | (empty) | MQTT password (optional) |
| `subscribe_topic` | `custom-service/in` | Topic to subscribe for incoming messages |
| `publish_topic` | `custom-service/out` | Topic to publish responses to |
| `client_id` | `custom-ha-service` | MQTT client identifier |

## Usage

### Sending Messages

Publish a message to the subscribe topic:
```bash
mosquitto_pub -h homeassistant -t custom-service/in -m '{"action":"test"}'
```

### Receiving Responses

The service will process the message and publish a response to the publish topic:
```json
{"status":"ok","received":{"action":"test"}}
```

Subscribe to receive responses:
```bash
mosquitto_sub -h homeassistant -t custom-service/out
```

## How It Works

1. **Startup**: The service starts automatically when Home Assistant boots (`boot: auto`)
2. **Connection**: Connects to the configured MQTT broker with automatic reconnection
3. **Subscription**: Subscribes to the configured topic with QoS 1 (At Least Once)
4. **Processing**: When messages arrive, they are processed and a JSON response is published
5. **Resilience**: If connection is lost, the service waits 5 seconds and reconnects automatically

## MQTT Topics

### Incoming (Subscribe)
- **Topic**: `custom-service/in` (configurable)
- **Purpose**: Send commands/data to the service
- **Format**: Any valid MQTT payload (JSON recommended)

### Outgoing (Publish)
- **Topic**: `custom-service/out` (configurable)
- **Purpose**: Receive responses from the service
- **Format**: JSON with status and received payload

## Logs

View logs in Home Assistant:
- Go to **Settings** → **Add-ons** → **Custom HA Service** → **Logs**

Or via command line:
```bash
ha logs addon_local_custom-ha-service
```

## Building for Production

To build the Docker image for HA Green (aarch64):

```bash
cd ha-addon
docker build --build-arg BUILD_ARCH=aarch64 -t custom-ha-service:aarch64 .
```

## Development

### Prerequisites

- Rust 1.75+
- Docker (for cross-compilation)
- MQTT broker (Mosquitto or Home Assistant MQTT addon)

### Project Structure

```
.
├── repository.yaml         # Home Assistant add-on repository configuration
├── Cargo.toml              # Rust project configuration
├── Cargo.lock              # Dependency lock file
├── src/
│   └── main.rs            # Main service implementation
├── custom-ha-service/      # Home Assistant add-on files
│   ├── config.yaml        # Add-on configuration
│   ├── Dockerfile         # Multi-arch Docker build
│   ├── build.yaml         # Build configuration
│   └── run.sh             # Container startup script
└── README.md
```

### Testing

1. Start an MQTT broker locally:
   ```bash
   docker run -d -p 1883:1883 eclipse-mosquitto
   ```

2. Run the service:
   ```bash
   cargo run
   ```

3. Test with mosquitto clients:
   ```bash
   # Subscribe to responses
   mosquitto_sub -t custom-service/out

   # Send a message (in another terminal)
   mosquitto_pub -t custom-service/in -m '{"test":"data"}'
   ```

## Integration Requirements

To use this custom service with your Home Assistant setup, ensure you have:

### Prerequisites

1. **MQTT Broker**: An MQTT broker must be running and accessible
   - **Option A**: Home Assistant MQTT Add-on (recommended)
     ```
     Settings → Add-ons → Add-on Store → MQTT → Install
     ```
   - **Option B**: External broker (Mosquitto, HiveMQ, etc.)
   - Default assumes broker at `homeassistant:1883`

2. **Network Access**: The add-on requires `host_network: true` to communicate with MQTT broker

3. **Compatible Hardware**: Designed for Home Assistant Green (aarch64), but supports all architectures

### Quick Integration Steps

1. **Install MQTT Integration** (if not already done):
   - Go to **Settings** → **Devices & Services** → **Add Integration**
   - Search for "MQTT" and configure with your broker details

2. **Configure the Service**:
   ```yaml
   mqtt_broker: homeassistant      # Or your broker IP/hostname
   mqtt_port: 1883
   mqtt_username: ""               # Leave empty if no auth
   mqtt_password: ""               # Leave empty if no auth
   subscribe_topic: custom-service/in
   publish_topic: custom-service/out
   client_id: custom-ha-service
   ```

3. **Test the Integration**:
   ```bash
   # From Home Assistant terminal or any MQTT client:
   mosquitto_pub -h homeassistant -t custom-service/in -m '{"action":"test"}'
   
   # Check response:
   mosquitto_sub -h homeassistant -t custom-service/out
   # Expected: {"status":"ok","received":{"action":"test"}}
   ```

4. **Use in Automations**:
   ```yaml
   # Example automation to send commands
   automation:
     - alias: "Test Custom Service"
       trigger:
         - platform: time
           at: "10:00:00"
       action:
         - service: mqtt.publish
           data:
             topic: custom-service/in
             payload: '{"command":"daily_report"}'
   ```

### MQTT Topics

- **Input**: `custom-service/in` - Send JSON commands here
- **Output**: `custom-service/out` - Receive responses here

### Troubleshooting Integration

| Issue | Solution |
|-------|----------|
| "Connection refused" | Verify MQTT broker is running on the configured host/port |
| "Authentication failed" | Check username/password in add-on configuration |
| No responses received | Ensure you're subscribed to `custom-service/out` topic |
| Service keeps restarting | Check logs: **Add-on → Logs** |

## License

MIT

## Troubleshooting

### Service won't start
- Check MQTT broker is running and accessible
- Verify MQTT credentials if authentication is enabled
- Check logs for connection errors

### No messages being processed
- Ensure topics are correctly configured
- Check that the MQTT broker allows anonymous connections (if no credentials)
- Verify network connectivity between addon and broker

### Reconnection loop
- Check MQTT broker is running
- Verify broker hostname/IP is correct
- Ensure no firewall blocks port 1883

## Deployment

### Docker Deployment

Build and run with Docker:

```bash
# Build Docker image
docker build -t custom-ha-service .

# Run with environment variables
docker run -e MQTT_BROKER=localhost \
           -e MQTT_PORT=1883 \
           -e SUBSCRIBE_TOPIC=custom-service/in \
           -e PUBLISH_TOPIC=custom-service/out \
           custom-ha-service
```

### Home Assistant Green Add-on Deployment

**Note:** This add-on is configured for **UTC+8 timezone** (Asia/Singapore, Hong Kong, etc.). 

The timezone is hardcoded in `src/processor.rs`:
```rust
const TIMEZONE_OFFSET_HOURS: i32 = 8;
```

To deploy as an add-on:

1. **Copy add-on files to HA Green**:
   ```bash
   # Copy the custom-ha-service folder to your HA Green
   scp -r custom-ha-service root@homeassistant.local:/addons/
   ```

2. **In Home Assistant**: Go to **Settings** → **Add-ons** → **Add-on Store**

3. **Reload**: Click **⋮** menu → **Check for updates**

4. **Install**: Find "Custom HA Service" in Local Add-ons → **Install**

5. **Configure**: Set MQTT broker settings in the add-on configuration

6. **Start**: Click **Start** to run the service

### Development Branch

**⚠️ This is a development branch.** Active development is happening here. For production use, switch to the `main` branch.

```bash
# View current branch
git branch

# Switch to development
git checkout development

# Switch to production-ready main
git checkout main
```

## Publishing to GitHub

To make this available as an open-source Home Assistant add-on:

1. **Create a GitHub Repository**:
   ```bash
   git init
   git add .
   git commit -m "Initial commit"
   git branch -M main
   git remote add origin https://github.com/YOUR_USERNAME/custom-ha-service.git
   git push -u origin main
   ```

2. **Update Repository URL**: Edit `repository.yaml` and replace `yourusername` with your actual GitHub username

3. **Users can now add your repository**:
   - Repository URL: `https://github.com/YOUR_USERNAME/custom-ha-service`
   - The add-on will appear in their Add-on Store automatically

## Support

For issues or feature requests, please open an issue on the project repository.
