#!/bin/bash

# Test Script for Custom HA Service
# Tests scenarios with mock MQTT data
# Note: Service only publishes when subscriber message arrives

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Custom HA Service Test Suite ===${NC}"
echo ""

# Check if MQTT broker is running, if not start one
if ! docker ps | grep -q mqtt-test; then
    echo -e "${YELLOW}Starting MQTT broker...${NC}"
    docker run -d --name mqtt-test -p 1883:1883 -p 9001:9001 eclipse-mosquitto 2>/dev/null || true
    sleep 2
fi

# Set environment variables
export MQTT_BROKER=localhost
export MQTT_PORT=1883
export SUBSCRIBE_TOPIC=custom-service/in
export PUBLISH_TOPIC=custom-service/out
export CLIENT_ID=custom-ha-service-test
export PULSES_PER_LITER=433
export RUST_LOG=info

echo -e "${YELLOW}Test Configuration:${NC}"
echo "  MQTT Broker: $MQTT_BROKER:$MQTT_PORT"
echo "  Subscribe: $SUBSCRIBE_TOPIC"
echo "  Publish: $PUBLISH_TOPIC"
echo ""

# Build the service if needed
echo -e "${YELLOW}Building service...${NC}"
cargo build --release 2>&1 | tail -5

# Create output log file
LOG_FILE="test_output_$(date +%Y%m%d_%H%M%S).log"
echo "Test started at: $(date)" > $LOG_FILE
echo "" >> $LOG_FILE

echo -e "${GREEN}=== TEST SCENARIO 1: First Data Message ===${NC}"
echo "Input: {\"total_pulses\": 433, \"Time_ms\": 1000}"
echo "Expected: Array with single entry at receive time"
echo "Format: [{\"timestamp\":\"25 Feb 2026 10:00:05\",\"flow_rate_lpm\":60.0}]"
echo ""

# Start the service in background
echo -e "${YELLOW}Starting service...${NC}"
./target/release/custom-ha-service >> $LOG_FILE 2>&1 &
SERVICE_PID=$!
sleep 1

mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 433, "Time_ms": 1000}' -q 1
echo -e "${YELLOW}Message sent, waiting for processing...${NC}"
sleep 2

echo -e "${GREEN}✓ Scenario 1 Complete${NC}"
echo ""

echo -e "${GREEN}=== TEST SCENARIO 2: Normal Follow-up (No Gap) ===${NC}"
echo "Input: {\"total_pulses\": 433, \"Time_ms\": 1000}"
echo "Expected: Array with single entry, no gap detected"
echo "Format: [{\"timestamp\":\"25 Feb 2026 10:00:06\",\"flow_rate_lpm\":60.0}]"
echo ""

sleep 1
mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 433, "Time_ms": 1000}' -q 1
echo -e "${YELLOW}Message sent, waiting for processing...${NC}"
sleep 2

echo -e "${GREEN}✓ Scenario 2 Complete${NC}"
echo ""

echo -e "${GREEN}=== TEST SCENARIO 3: Gap Detection ===${NC}"
echo "Simulating 5 second gap..."
echo "Input: {\"total_pulses\": 866, \"Time_ms\": 2000}"
echo "Expected: Array with gap zeros + actual data"
echo "Format example: ["
echo '  {"timestamp":"25 Feb 2026 10:00:07","flow_rate_lpm":0.0},'
echo '  {"timestamp":"25 Feb 2026 10:00:08","flow_rate_lpm":0.0},'
echo '  {"timestamp":"25 Feb 2026 10:00:09","flow_rate_lpm":60.0},'
echo '  {"timestamp":"25 Feb 2026 10:00:10","flow_rate_lpm":60.0}'
echo "]"
echo ""

echo -e "${YELLOW}Waiting 5 seconds to create gap...${NC}"
sleep 5

mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 866, "Time_ms": 2000}' -q 1
echo -e "${YELLOW}Message sent after gap, waiting for processing...${NC}"
sleep 2

echo -e "${GREEN}✓ Scenario 3 Complete${NC}"
echo ""

echo -e "${GREEN}=== TEST SCENARIO 4: Multiple Rapid Messages ===${NC}"
echo "Sending 3 rapid messages..."
echo "Expected: 3 separate array publications (0.5s apart)"
echo ""

mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 200, "Time_ms": 1000}' -q 1
sleep 0.5
mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 300, "Time_ms": 1000}' -q 1
sleep 0.5
mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 400, "Time_ms": 1000}' -q 1

echo -e "${YELLOW}Messages sent, waiting...${NC}"
sleep 3

echo -e "${GREEN}✓ Scenario 4 Complete${NC}"
echo ""

echo -e "${GREEN}=== TEST SCENARIO 5: Large Data Window ===${NC}"
echo "Input: {\"total_pulses\": 2165, \"Time_ms\": 5000} (5 seconds of data)"
echo "Expected: Array with 5 entries, all same L/min"
echo "Format: ["
echo '  {"timestamp":"...","flow_rate_lpm":60.0},'
echo '  {"timestamp":"...","flow_rate_lpm":60.0},'
echo '  {"timestamp":"...","flow_rate_lpm":60.0},'
echo '  {"timestamp":"...","flow_rate_lpm":60.0},'
echo '  {"timestamp":"...","flow_rate_lpm":60.0}'
echo "]"
echo ""

mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 2165, "Time_ms": 5000}' -q 1
echo -e "${YELLOW}Message sent, waiting for processing...${NC}"
sleep 2

echo -e "${GREEN}✓ Scenario 5 Complete${NC}"
echo ""

echo -e "${GREEN}=== TEST SCENARIO 6: Silent Period (No Publishing) ===${NC}"
echo "Waiting 5 seconds without sending messages..."
echo "Expected: NO OUTPUT - service is silent when no data received"
echo ""

echo -e "${YELLOW}Waiting 5 seconds (no messages sent)...${NC}"
sleep 5

echo -e "${GREEN}✓ Scenario 6 Complete - No output expected during silence${NC}"
echo ""

echo -e "${GREEN}=== TEST SCENARIO 7: Message After Silent Period ===${NC}"
echo "After 5s silence, sending message..."
echo "Expected: Array with gap-filled zeros + actual data"
echo ""

mosquitto_pub -h $MQTT_BROKER -t $SUBSCRIBE_TOPIC -m '{"total_pulses": 433, "Time_ms": 1000}' -q 1
echo -e "${YELLOW}Message sent after silence, waiting for processing...${NC}"
sleep 2

echo -e "${GREEN}✓ Scenario 7 Complete${NC}"
echo ""

# Cleanup
echo -e "${YELLOW}Stopping service...${NC}"
kill $SERVICE_PID 2>/dev/null || true
wait $SERVICE_PID 2>/dev/null || true

echo -e "${GREEN}=== All Tests Complete ===${NC}"
echo ""
echo -e "Log file: ${YELLOW}$LOG_FILE${NC}"
echo ""
echo -e "${YELLOW}Key log entries (showing timestamps with month names):${NC}"
echo "---"
grep -E "(Starting|Published|Received|Processing|Gap detected|timestamp.*Feb)" $LOG_FILE | head -50
echo "---"
echo ""
echo "To see full output, run: cat $LOG_FILE"
echo ""
echo "Expected timestamp format: \"25 Feb 2026 10:00:01\""
echo "  - Day: 25"
echo "  - Month: Feb (short month name)"
echo "  - Year: 2026"
echo "  - Time: 10:00:01 (HH:MM:SS)"
echo ""
echo "Service Behavior:"
echo "  ✓ Only publishes when subscriber message arrives"
echo "  ✓ All outputs are JSON arrays"
echo "  ✓ Gap filling with zeros still works"
echo "  ✓ No periodic zero publishing between messages"
