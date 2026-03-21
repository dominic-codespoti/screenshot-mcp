# screenshot-mcp

A robust Model Context Protocol (MCP) server written in Rust that enables Large Language Models (LLMs) to natively take screenshots across operating systems (Linux, Windows, macOS). 

Exposes two tools:
- `list_screenshot_targets`: Get information about all active monitors and windows, including their IDs, process names, and window titles.
- `take_screenshot`: Takes the screenshot and returns the image inline as a base64-encoded standard MCP Image block. Supports screenshotting by `monitor`, `window`, and even `pid` (discovering the window dynamically by process ID, inclusive of child processes).

## Installation

```bash
cargo install screenshot-mcp
```

## Adding to Cursor / Claude 

In your MCP client configurations, just point to the binary:
```json
{
  "mcpServers": {
    "screenshot-mcp": {
      "command": "screenshot-mcp"
    }
  }
}
```

