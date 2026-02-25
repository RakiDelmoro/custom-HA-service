# Water Flow MQTT Service - Implementation Guide

## Overview
This service processes flow sensor data from MQTT, calculates L/min (liters per minute), and handles gap filling for smooth visualization in Home Assistant.

## Architecture

### Module Structure
```
src/
├── main.rs          # Entry point, MQTT event loop
├── config.rs        # Environment variables and configuration
├── models.rs        # Data structures (SensorData, OutputMessage)
├── processor.rs     # Core logic (L/min calculation, gap filling)
├── state.rs         # In-memory state management
└── test_scenarios.sh # Test scenarios documentation
```

## Processing Logic

### 1. Receive MQTT Message
```json
{"total_pulses": 80, "Time_ms": 10000}
```

### 2. Calculate L/min
Formula: `(pulses / seconds) × (60 / pulses_per_liter)`
- 433 pulses = 1 liter (configurable)
- Example: 80 pulses in 10 seconds = 8 pulses/sec
- L/min: 8 × (60/433) = 1.109 L/min

### 3. Handle Scenarios

#### Scenario A: First Message
- Parse and calculate
- Store L/min in state
- Return single value

#### Scenario B: Normal 1-second Interval
- Calculate L/min
- Update state
- Return single value

#### Scenario C: Burst (>1 second)
- Calculate L/min
- Create N values (N = seconds)
- All values same L/min
- Batch publish

#### Scenario D: Gap Detected
- Calculate gap seconds (time since last message > threshold)
- Fill gap with configured mode (0 or last-known)
- Add new data values
- Batch publish gap + new data

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MQTT_BROKER` | `192.168.50.47` | MQTT broker IP |
| `MQTT_PORT` | `1883` | MQTT broker port |
| `MQTT_USERNAME` | (none) | MQTT username |
| `MQTT_PASSWORD` | (none) | MQTT password |
| `SUBSCRIBE_TOPIC` | `custom-service/in` | Input topic from sensor |
| `PUBLISH_TOPIC` | `custom-service/out` | Output topic to HA |
| `PULSES_PER_LITER` | `433` | Sensor calibration |
| `GAP_FILL_MODE` | `last` | `last` or `zero` |
| `GAP_THRESHOLD_MS` | `2000` | Gap detection threshold (2 seconds) |
| `RUST_LOG` | `info` | Logging level |

### Gap Fill Modes
- `last`: Fill gaps with last known L/min value (smooth line)
- `zero`: Fill gaps with 0 L/min (shows gaps in graph)

## Input/Output Format

### Input (from sensor)
```json
{
  "total_pulses": 80,
  "Time_ms": 10000
}
```

### Output (to Home Assistant)
```json
{
  "timeseries": [1.109, 1.109, 1.109, 1.109, 1.109],
  "metadata": {
    "samples": 5,
    "time_span_seconds": 5,
    "avg_l_per_min": 1.109,
    "pulses_per_liter": 433,
    "gap_filled": false
  }
}
```

## Test Scenarios

### Scenario 1: Normal 1-second interval
```bash
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in \
  -m '{"total_pulses":8,"Time_ms":1000}'
```
**Output:** `[1.109]` (single value)

### Scenario 2: Burst data (10 seconds)
```bash
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in \
  -m '{"total_pulses":80,"Time_ms":10000}'
```
**Output:** `[1.109, 1.109, 1.109, 1.109, 1.109, 1.109, 1.109, 1.109, 1.109, 1.109]` (10 values)

### Scenario 3: Gap + burst
```bash
# First message
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in -m '{"total_pulses":8,"Time_ms":1000}'

# Wait 5 seconds...

# Second message (gap detected)
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in -m '{"total_pulses":40,"Time_ms":4000}'
```
**Output:** `[1.109, 1.109, 1.109, 1.109, 1.109, 1.386, 1.386, 1.386, 1.386]`
- 5 gap fill values (last known: 1.109)
- 4 new values (1.386 L/min)

### Scenario 4: Sensor initialization (powered by water)
**Context:** Sensor is powered by water flow. When water starts, sensor powers on and needs ~10 seconds to setup WiFi/MQTT. During setup, no data is published.

```bash
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in \
  -m '{"total_pulses":0,"Time_ms":10091}'
```
**Output:** `[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]` (10 values)
- All zeros: no flow detected during 10 second setup
- Service initialized with `last_l_per_min = 0.0`
- Subsequent messages will use 0.0 for gap fill (if GAP_FILL_MODE=last)

**Next message after initialization:**
```bash
# Water flowing, 5 seconds, 40 pulses
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in \
  -m '{"total_pulses":40,"Time_ms":5000}'
```
**Output:** `[0.0, 0.0, 0.0, 0.0, 0.0, 1.109, 1.109, 1.109, 1.109, 1.109]`
- First 5 values: gap fill (initialized with 0.0)
- Last 5 values: actual flow (1.109 L/min)

## Running the Service

### 1. Set Environment Variables
```bash
export MQTT_BROKER=192.168.50.47
export MQTT_PORT=1883
export MQTT_USERNAME=mqtt_indicator_1
export MQTT_PASSWORD=mqtt
export SUBSCRIBE_TOPIC=custom-service/in
export PUBLISH_TOPIC=custom-service/out
export PULSES_PER_LITER=433
export GAP_FILL_MODE=last
export GAP_THRESHOLD_MS=2000
export RUST_LOG=info
```

### 2. Build and Run
```bash
# Build release version
cargo build --release

# Run the service
./target/release/custom-ha-service
```

### 3. Subscribe to Output
```bash
mosquitto_sub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/out
```

### 4. Send Test Messages
```bash
# Test with different scenarios
mosquitto_pub -h 192.168.50.47 -p 1883 -u mqtt_indicator_1 -P mqtt \
  -t custom-service/in \
  -m '{"total_pulses":8,"Time_ms":1000}'
```

## Features

✅ Modular architecture (6 files)
✅ L/min calculation from pulses
✅ Per-second slicing for burst data
✅ Gap detection and filling
✅ Configurable gap fill mode (0 or last-known)
✅ Batch publishing for gaps + new data
✅ Thread-safe state management
✅ Automatic MQTT reconnection
✅ Comprehensive logging

## Next Steps for Phase 2 (Open Source)

1. **Add unit tests** (some already in processor.rs)
2. **Add integration tests**
3. **Create Dockerfile** for HA add-on
4. **Add icon/logo** for HA add-on
5. **Write README.md** for GitHub
6. **Choose license** (MIT recommended)
7. **Create GitHub repository**
8. **Build multi-arch Docker images**
9. **Test on actual Home Assistant**

## Notes

- The service keeps running as long as HA is running
- State is in-memory (lost on restart) - intentional for simplicity
- Gap filling provides smooth visualization
- Burst data is sliced per second for detailed graphs
- No maximum gap limit - fills all detected gaps
