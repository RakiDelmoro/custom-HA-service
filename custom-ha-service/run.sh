#!/usr/bin/with-contenv bashio

# Home Assistant addon configuration
set -e

bashio::log.info "Starting Custom HA Service..."

# Export configuration from Home Assistant addon options
export MQTT_BROKER="$(bashio::config 'mqtt_broker')"
export MQTT_PORT="$(bashio::config 'mqtt_port')"
export MQTT_USERNAME="$(bashio::config 'mqtt_username')"
export MQTT_PASSWORD="$(bashio::config 'mqtt_password')"
export SUBSCRIBE_TOPIC="$(bashio::config 'subscribe_topic')"
export PUBLISH_TOPIC="$(bashio::config 'publish_topic')"
export CLIENT_ID="$(bashio::config 'client_id')"
export PULSES_PER_LITER="$(bashio::config 'pulses_per_liter')"

bashio::log.info "MQTT Broker: ${MQTT_BROKER}:${MQTT_PORT}"
bashio::log.info "Subscribe Topic: ${SUBSCRIBE_TOPIC}"
bashio::log.info "Publish Topic: ${PUBLISH_TOPIC}"

# Run the service
exec /usr/local/bin/custom-ha-service
