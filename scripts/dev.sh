#!/bin/bash

# CookieCommander Development Script
# Cleans, builds, tests, and runs the application in development mode

# Ensure script execution stops on errors
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check options
CLEAN=false
RUN_TESTS=true
SAVE_LOGS=false
FORCE_QUIT=false
LOG_LEVEL=""

show_help() {
    echo -e "${BLUE}CookieCommander Development Script${NC}"
    echo ""
    echo "Usage: ./scripts/dev.sh [options]"
    echo ""
    echo "Options:"
    echo "  --clean                 Run cargo clean before building"
    echo "  --no-test               Skip running tests before starting"
    echo "  --save-logs             Save build, test, and runtime stderr to logs/"
    echo "  --force-quit            Terminate existing cookie_commander processes before starting"
    echo "  --log-level <level>     Set the Rust log level (error, warn, info, debug, trace)"
    echo "  --help                  Show this help message"
    echo ""
    echo "Example:"
    echo "  ./scripts/dev.sh --clean --log-level debug"
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --no-test)
            RUN_TESTS=false
            shift
            ;;
        --save-logs)
            SAVE_LOGS=true
            shift
            ;;
        --force-quit)
            FORCE_QUIT=true
            shift
            ;;
        --log-level)
            if [ -n "$2" ] && [ ${2:0:1} != "-" ]; then
                LOG_LEVEL="$2"
                shift 2
            else
                echo -e "${RED}Error: Argument for --log-level is missing${NC}" >&2
                exit 1
            fi
            ;;
        --help|help|-h)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Navigate to project root
cd "$PROJECT_ROOT"

# Set Rust logging environment variable if specified
if [ -n "$LOG_LEVEL" ]; then
    export RUST_LOG="$LOG_LEVEL"
    echo -e "${BLUE}ℹ️  RUST_LOG set to: $RUST_LOG${NC}"
fi

# Set up logging if requested
if [ "$SAVE_LOGS" = true ]; then
    LOG_DIR="$PROJECT_ROOT/logs"
    mkdir -p "$LOG_DIR"
    TIMESTAMP=$(date "+%Y-%m-%d_%H-%M-%S")
    
    BUILD_LOG="$LOG_DIR/${TIMESTAMP}_dev_build.log"
    TEST_LOG="$LOG_DIR/${TIMESTAMP}_dev_test.log"
    RUNTIME_LOG="$LOG_DIR/${TIMESTAMP}_dev_runtime.log"
    
    echo -e "${BLUE}📝 Logs will be saved to:${NC}"
    echo -e "   Build:   ${BUILD_LOG}"
    echo -e "   Test:    ${TEST_LOG}"
    echo -e "   Runtime: ${RUNTIME_LOG}"
fi

# Function to cleanup on exit (restores terminal state if necessary)
cleanup() {
    exit 0
}
trap cleanup SIGINT SIGTERM EXIT

# Force quit existing instances of cookie_commander if requested
if [ "$FORCE_QUIT" = true ]; then
    echo -e "${YELLOW}🔍 Checking for existing cookie_commander processes...${NC}"
    # Find PIDs of processes matching "cookie_commander"
    PIDS=$(pgrep -x "cookie_commander" || true)
    
    if [ -n "$PIDS" ]; then
        echo -e "${YELLOW}🛑 Found existing processes (PIDs: $PIDS). Terminating...${NC}"
        for PID in $PIDS; do
            echo -e "${YELLOW}   Killing PID $PID...${NC}"
            kill "$PID" 2>/dev/null || true
        done
        sleep 1
        
        # Verify if they are still running
        REMAINING_PIDS=$(pgrep -x "cookie_commander" || true)
        if [ -n "$REMAINING_PIDS" ]; then
             echo -e "${RED}⚠️  Some processes did not exit gracefully. Force killing...${NC}"
             kill -9 $REMAINING_PIDS 2>/dev/null || true
             sleep 1
        else
             echo -e "${GREEN}✅ Existing processes terminated.${NC}"
        fi
    else
        echo -e "${GREEN}✅ No existing cookie_commander processes found.${NC}"
    fi
fi

# Step 1: Clean (if requested)
if [ "$CLEAN" = true ]; then
    echo -e "${GREEN}▶ Cleaning workspace...${NC}"
    if [ "$SAVE_LOGS" = true ]; then
        cargo clean > "$BUILD_LOG" 2>&1
    else
        cargo clean
    fi
    echo -e "${GREEN}✅ Clean complete.${NC}"
fi

# Step 1.5: Auto-increment version build/patch number
echo -e "${GREEN}▶ Auto-incrementing build version...${NC}"
NEW_VER=$("$SCRIPT_DIR/increment_version.sh" patch)
echo -e "${BLUE}ℹ️  New version: $NEW_VER${NC}"

# Step 2: Build
echo -e "${GREEN}▶ Building project in development mode...${NC}"
if [ "$SAVE_LOGS" = true ]; then
    cargo build 2>&1 | tee -a "$BUILD_LOG"
else
    cargo build
fi
echo -e "${GREEN}✅ Build complete.${NC}"

# Step 3: Test
if [ "$RUN_TESTS" = true ]; then
    echo -e "${GREEN}▶ Running tests...${NC}"
    if [ "$SAVE_LOGS" = true ]; then
        cargo test 2>&1 | tee "$TEST_LOG"
    else
        cargo test
    fi
    echo -e "${GREEN}✅ Tests completed successfully.${NC}"
fi

# Step 4: Run
echo -e "${GREEN}▶ Running CookieCommander...${NC}"
if [ "$SAVE_LOGS" = true ]; then
    cargo run 2> "$RUNTIME_LOG"
else
    cargo run
fi
