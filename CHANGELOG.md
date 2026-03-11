# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-24

### Added
- Initial production release of FlowPulse MQTT
- Convert pulse sensor data to water flow rates (L/min) via MQTT
- Core processing engine with gap detection and filling
- Duplicate timestamp prevention
- Support for burst data (multi-second intervals)
- Configurable pulses per liter calibration
- Home Assistant add-on integration
- Comprehensive test suite (11 tests)
- Multi-architecture Docker build support (aarch64, amd64, armhf, armv7, i386)
- In-memory MQTT mock for testing

### Features
- Gap filling (configurable mode: last known or zero)
- Automatic MQTT reconnection
- Chronological output (oldest to newest)
- Silent operation between messages
- Per-second data slicing for burst inputs
- Timezone-aware timestamps (UTC+8)

[0.1.0]: https://github.com/RakiDelmoro/custom-ha-service/releases/tag/v0.1.0
