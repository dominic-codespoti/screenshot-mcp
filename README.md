# screenshot-mcp

A robust Model Context Protocol (MCP) server written in Rust that enables Large Language Models (LLMs) to natively take screenshots across operating systems (Linux, Windows, macOS). 

Exposes two tools:
- `list_screenshot_targets`: Get information about all active monitors and windows, including their IDs, process names, and window titles.
- `take_screenshot`: Takes the screenshot and returns the image inline as a base64-encoded standard MCP Image block. Supports screenshotting by `monitor`, `window`, and even `pid` (discovering the window dynamically by process ID, inclusive of child processes).

## Installation

```bash
cargo install screenshot-mcp
```

## Adding to AI Clients

Because this server interacts purely through the standardized `stdio` MCP transport, you can plug it into any compatible AI assistant. The configuration uses `sh -c "cargo install screenshot-mcp && screenshot-mcp"` as its launch command, ensuring that the crate is safely and idempotently downloaded & executed from `~/.cargo/bin`.

### Claude Desktop

1. Open your configuration file:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
2. Add the server to the `mcpServers` object:

```json
{
  "mcpServers": {
    "screenshot-mcp": {
      "command": "sh",
      "args": ["-c", "cargo install screenshot-mcp && screenshot-mcp"]
    }
  }
}
```

### Cursor

1. Open Cursor Settings (`Cmd/Ctrl + Shift + J`).
2. Go to **Features** > **MCP Servers**.
3. Click **+ Add new MCP server** and set:
   - Type: `command`
   - Name: `screenshot-mcp`
   - Command: `sh -c "cargo install screenshot-mcp && screenshot-mcp"`

### GitHub Copilot (VS Code)

1. Open your VS Code `settings.json`.
2. Add the server to Copilot's experimental MCP mapping:

```json
{
  "github.copilot.chat.experimental.mcp.servers": {
    "screenshot-mcp": {
      "command": "sh",
      "args": ["-c", "cargo install screenshot-mcp && screenshot-mcp"]
    }
  }
}
```
*Note: Make sure to restart your editor or AI client after adding the configuration for the server to spin up correctly. The very first time it runs, Cargo will compile the binary, which may take ~30-60 seconds. Subsequent boots will be practically instant!*

