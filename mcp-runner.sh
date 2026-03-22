#!/bin/bash
export RUST_LOG=debug
exec /home/dom/.cargo/bin/screenshot-mcp 2>> /tmp/mcp-server-stderr.log
