#!/usr/bin/env bash
set -euo pipefail

MISSING=0

check_tool() {
    local tool="$1"
    local hint="$2"
    if ! command -v "$tool" &>/dev/null; then
        echo "  MISSING: $tool — $hint"
        MISSING=1
    fi
}

echo "Checking prerequisites..."

check_tool docker "Install Docker Desktop or docker engine"
check_tool curl "Should be pre-installed on most systems"
check_tool jq "brew install jq / apt install jq"
check_tool wrk "brew install wrk / apt install wrk"
check_tool pandoc "brew install pandoc / apt install pandoc"
check_tool bc "brew install bc / apt install bc"

if command -v docker &>/dev/null; then
    if ! docker compose version &>/dev/null; then
        echo "  MISSING: docker compose v2 plugin — update Docker or install compose plugin"
        MISSING=1
    fi
    if ! docker info &>/dev/null 2>&1; then
        echo "  ERROR: Docker daemon is not running"
        MISSING=1
    fi
fi

if [ "$MISSING" -eq 1 ]; then
    echo "Install missing tools and try again."
    exit 1
fi

echo "All prerequisites satisfied."
