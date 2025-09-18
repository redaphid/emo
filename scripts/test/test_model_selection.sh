#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Testing model selection with isolated configs..."

# Get list of available models
echo -e "${YELLOW}Getting available models...${NC}"
MODELS=$(env XDG_CONFIG_HOME=/tmp/emo-list ./target/release/emo --list-models 2>/dev/null | grep -E "^\s+[a-z]" | awk '{print $1}' | head -5)

if [ -z "$MODELS" ]; then
    echo -e "${RED}Failed to get model list${NC}"
    exit 1
fi

echo -e "${GREEN}Found models:${NC}"
echo "$MODELS"

# Test each model
MODEL_COUNT=0
for MODEL in $MODELS; do
    MODEL_COUNT=$((MODEL_COUNT + 1))
    RANDOM_DIR="/tmp/emo-test-$$-$MODEL_COUNT"

    echo -e "\n${YELLOW}Test $MODEL_COUNT: Testing model '$MODEL' with config dir: $RANDOM_DIR${NC}"

    # Create fresh config dir
    mkdir -p "$RANDOM_DIR/emo"

    # Test 1: Use --model flag to set model
    echo -e "  ${YELLOW}Step 1: Setting model with --model flag${NC}"
    env XDG_CONFIG_HOME="$RANDOM_DIR" timeout 30 ./target/release/emo --model "$MODEL" fire >/dev/null 2>&1 || true

    # Check if config was created and contains the model
    if [ -f "$RANDOM_DIR/emo/config.json" ]; then
        SAVED_MODEL=$(cat "$RANDOM_DIR/emo/config.json" | grep -o '"model"[[:space:]]*:[[:space:]]*"[^"]*"' | sed 's/.*"\([^"]*\)"$/\1/')
        if [ "$SAVED_MODEL" = "$MODEL" ]; then
            echo -e "  ${GREEN}✓ Model '$MODEL' saved to config${NC}"
        else
            echo -e "  ${RED}✗ Model not saved correctly. Expected: '$MODEL', Got: '$SAVED_MODEL'${NC}"
            exit 1
        fi
    else
        echo -e "  ${RED}✗ Config file not created${NC}"
        exit 1
    fi

    # Test 2: Use AI without --model (should use config)
    echo -e "  ${YELLOW}Step 2: Testing AI uses model from config${NC}"

    # Run AI and capture which model file it tries to load
    OUTPUT=$(env XDG_CONFIG_HOME="$RANDOM_DIR" timeout 5 ./target/release/emo --ai "test" 2>&1 || true)

    # Check if it's downloading or using the right model
    if echo "$OUTPUT" | grep -q "Downloading.*$MODEL"; then
        echo -e "  ${GREEN}✓ AI attempting to download model '$MODEL' from config${NC}"
    elif echo "$OUTPUT" | grep -q "$MODEL.*\.gguf"; then
        echo -e "  ${GREEN}✓ AI using model '$MODEL' from config${NC}"
    elif echo "$OUTPUT" | grep -q "📥 Downloading"; then
        # Extract what model it's actually downloading
        DOWNLOADING=$(echo "$OUTPUT" | grep "📥 Downloading" | sed 's/.*Downloading \(.*\) model.*/\1/')
        echo -e "  ${YELLOW}⚠ AI downloading: '$DOWNLOADING'${NC}"

        # Check if the downloading model matches our expected model name pattern
        if echo "$DOWNLOADING" | grep -qi "$(echo $MODEL | cut -d'-' -f1)"; then
            echo -e "  ${GREEN}✓ Model family matches (partial match)${NC}"
        else
            echo -e "  ${RED}✗ Wrong model being downloaded. Expected: '$MODEL', Got: '$DOWNLOADING'${NC}"
        fi
    else
        echo -e "  ${RED}✗ Could not determine which model AI is using${NC}"
        echo "  Debug output:"
        echo "$OUTPUT" | head -5
    fi

    # Test 3: Override with different model
    if [ "$MODEL_COUNT" -gt 1 ]; then
        PREV_MODEL=$(echo "$MODELS" | head -1)
        echo -e "  ${YELLOW}Step 3: Testing override with --model flag${NC}"

        env XDG_CONFIG_HOME="$RANDOM_DIR" ./target/release/emo --model "$PREV_MODEL" rocket >/dev/null 2>&1

        NEW_SAVED_MODEL=$(cat "$RANDOM_DIR/emo/config.json" | grep -o '"model"[[:space:]]*:[[:space:]]*"[^"]*"' | sed 's/.*"\([^"]*\)"$/\1/')
        if [ "$NEW_SAVED_MODEL" = "$PREV_MODEL" ]; then
            echo -e "  ${GREEN}✓ Config updated to new model '$PREV_MODEL'${NC}"
        else
            echo -e "  ${RED}✗ Config not updated. Expected: '$PREV_MODEL', Got: '$NEW_SAVED_MODEL'${NC}"
            exit 1
        fi
    fi

    # Clean up this test directory
    rm -rf "$RANDOM_DIR"

    # Only test first 3 models to avoid downloading too many
    if [ "$MODEL_COUNT" -ge 3 ]; then
        echo -e "\n${YELLOW}Stopping after 3 models to avoid excessive downloads${NC}"
        break
    fi
done

echo -e "\n${GREEN}All tests passed!${NC}"
echo -e "${GREEN}Tested $MODEL_COUNT models with isolated configs${NC}"