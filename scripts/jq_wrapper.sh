#!/bin/bash

# Wrapper script for jq with proper path resolution

JQ_PATH="/c/Users/kanal/AppData/Local/Microsoft/WinGet/Packages/jqlang.jq_Microsoft.WinGet.Source_8wekyb3d8bbwe/jq.exe"

if [ -f "$JQ_PATH" ]; then
    "$JQ_PATH" "$@"
else
    echo "jq not found at $JQ_PATH" >&2
    exit 127
fi