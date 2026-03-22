use async_trait::async_trait;
use base64::Engine;
use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    error::SdkResult,
    macros,
    mcp_server::{McpServerOptions, ServerHandler, ToMcpServerHandler, server_runtime},
    schema::*,
};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use xcap::{Monitor, Window};
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::{fmt::writer::MakeWriterExt, EnvFilter};

#[derive(Debug)]
struct SimpleError(String);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}

fn make_error(msg: impl Into<String>) -> CallToolError {
    CallToolError::new(SimpleError(msg.into()))
}

// Define MCP tools

#[macros::mcp_tool(
    name = "list_screenshot_targets",
    description = "List all available monitors and windows that can be screenshot. Returns their IDs, PIDs, and names."
)]
#[derive(Debug, Deserialize, Serialize, macros::JsonSchema)]
pub struct ListScreenshotTargetsTool {}

#[macros::mcp_tool(
    name = "take_screenshot",
    description = "Take a screenshot of a specific monitor, window, or window matching a process PID."
)]
#[derive(Debug, Deserialize, Serialize, macros::JsonSchema)]
pub struct TakeScreenshotTool {
    /// The type of target to screenshot. Allowed values: "monitor", "window", "pid", "primary_monitor", "all_monitors"
    pub target_type: String,

    /// The ID of the monitor or window, or the PID of the process. Not required if target is primary_monitor or all_monitors.
    pub target_id: Option<String>,

    /// Optional absolute path to save the screenshot to disk. Useful for editors that cannot parse or visualize raw SAS urls or base64 images directly.
    pub save_path: Option<String>,
}

// Define a custom handler
#[derive(Default)]
struct ScreenshotHandler;

// Implement ServerHandler
#[async_trait]
impl ServerHandler for ScreenshotHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: std::sync::Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: vec![
                ListScreenshotTargetsTool::tool(),
                TakeScreenshotTool::tool(),
            ],
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: std::sync::Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "list_screenshot_targets" => {
                let mut output = String::new();

                output.push_str("=== Monitors ===\n");
                let monitors = Monitor::all().unwrap_or_default();
                for monitor in monitors {
                    let id = monitor.id().unwrap_or(0);
                    let name = monitor.name().unwrap_or_else(|_| "Unknown".to_string());
                    let is_primary = monitor.is_primary().unwrap_or(false);
                    output.push_str(&format!(
                        "ID: {}, Name: {}, Primary: {}\n",
                        id, name, is_primary
                    ));
                }

                output.push_str("\n=== Windows ===\n");
                let windows = Window::all().unwrap_or_default();
                for window in windows {
                    if window.is_minimized().unwrap_or(true) {
                        continue;
                    }
                    let id = window.id().unwrap_or(0);
                    let pid = window.pid().unwrap_or(0);
                    let app = window.app_name().unwrap_or_else(|_| "Unknown".to_string());
                    let title = window.title().unwrap_or_else(|_| "Unknown".to_string());
                    output.push_str(&format!(
                        "ID: {}, PID: {}, App: {}, Title: '{}'\n",
                        id, pid, app, title
                    ));
                }

                Ok(CallToolResult {
                    content: vec![ContentBlock::text_content(output)],
                    is_error: Some(false),
                    meta: None,
                    structured_content: None,
                })
            }
            "take_screenshot" => {
                let args = params.arguments.unwrap_or_default();
                let tool_args: TakeScreenshotTool =
                    serde_json::from_value(serde_json::Value::Object(args.clone()))
                        .map_err(|e| make_error(format!("Invalid arguments: {}", e)))?;

                let target_type = tool_args.target_type.as_str();
                let mut captured_image = None;

                match target_type {
                    "primary_monitor" => {
                        if let Ok(monitors) = Monitor::all() {
                            if let Some(monitor) = monitors
                                .into_iter()
                                .find(|m| m.is_primary().unwrap_or(false))
                            {
                                captured_image = monitor.capture_image().ok();
                            }
                        }
                    }
                    "all_monitors" => {
                        return Err(make_error(
                            "all_monitors not yet implemented, please specify a monitor or use primary_monitor",
                        ));
                    }
                    "monitor" => {
                        let id = tool_args
                            .target_id
                            .ok_or_else(|| make_error("target_id required for monitor"))?;
                        let id_u32 = id.parse::<u32>().unwrap_or_default();

                        if let Ok(monitors) = Monitor::all() {
                            if let Some(monitor) =
                                monitors.into_iter().find(|m| m.id().unwrap_or(0) == id_u32)
                            {
                                captured_image = monitor.capture_image().ok();
                            }
                        }
                    }
                    "window" => {
                        let id = tool_args
                            .target_id
                            .ok_or_else(|| make_error("target_id required for window"))?;
                        let id_u32 = id.parse::<u32>().unwrap_or_default();

                        if let Ok(windows) = Window::all() {
                            if let Some(window) =
                                windows.into_iter().find(|w| w.id().unwrap_or(0) == id_u32)
                            {
                                captured_image = window.capture_image().ok();
                            }
                        }
                    }
                    "pid" => {
                        let pid_str = tool_args
                            .target_id
                            .ok_or_else(|| make_error("target_id required for pid"))?;
                        let target_pid = pid_str.parse::<u32>().unwrap_or_default();

                        // Use sysinfo to find the process and its children just in case the launcher daemonized
                        let mut system = System::new();
                        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

                        let mut valid_pids = std::collections::HashSet::new();
                        valid_pids.insert(target_pid);

                        // Rudimentary child process collection
                        let mut pids_to_check = vec![target_pid];
                        while let Some(current_pid) = pids_to_check.pop() {
                            let sys_pid = sysinfo::Pid::from_u32(current_pid);
                            for (p, proc) in system.processes() {
                                if proc.parent() == Some(sys_pid) {
                                    let child_pid = p.as_u32();
                                    if valid_pids.insert(child_pid) {
                                        pids_to_check.push(child_pid);
                                    }
                                }
                            }
                        }

                        if let Ok(windows) = Window::all() {
                            if let Some(window) = windows.into_iter().find(|w| {
                                let w_pid = w.pid().unwrap_or(0);
                                valid_pids.contains(&w_pid) && !w.is_minimized().unwrap_or(true)
                            }) {
                                captured_image = window.capture_image().ok();
                            }
                        }
                    }
                    _ => return Err(make_error(format!("Unknown target_type: {}", target_type))),
                }

                match captured_image {
                    Some(image) => {
                        use std::io::Cursor;
                        let mut buffer = Cursor::new(Vec::new());
                        if image
                            .write_to(&mut buffer, image::ImageFormat::Png)
                            .is_err()
                        {
                            return Err(make_error("Failed to encode image to PNG"));
                        }

                        let mut contents = Vec::new();

                        // If save_path was provided, save the image directly to disk
                        if let Some(path) = &tool_args.save_path {
                            if let Err(e) = image.save(path) {
                                return Err(make_error(format!("Failed to save image to disk {}: {}", path, e)));
                            }
                            contents.push(ContentBlock::text_content(
                                format!("Screenshot successfully saved to disk at: {}", path),
                            ));
                        }

                        let base64_image =
                            base64::engine::general_purpose::STANDARD.encode(buffer.into_inner());

                        contents.push(ContentBlock::image_content(
                            base64_image,
                            "image/png".to_string(),
                        ));

                        Ok(CallToolResult {
                            content: contents,
                            is_error: Some(false),
                            meta: None,
                            structured_content: None,
                        })
                    }
                    None => Err(make_error(
                        "Could not capture screenshot: target not found or capture failed",
                    )),
                }
            }
            unknown => Err(make_error(format!("Unknown tool: {}", unknown))),
        }
    }
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    // Set up logging to a file so it doesn't pollute stdout
    let file_appender = rolling::daily("/tmp", "screenshot-mcp.log");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(file_appender)
        .with_ansi(false)
        .init();

    info!("Starting screenshot-mcp server...");

    // Define server details and capabilities
    let server_info = InitializeResult {
        server_info: Implementation {
            name: "screenshot-mcp".into(),
            version: "0.1.0".into(),
            title: Some("Screenshot MCP Server".to_string()),
            description: Some("MCP server for taking screenshots across platforms".to_string()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        // We set the protocol version high enough to satisfy Copilot's non-standard `2025-11-25` string comparisons
        // while also gracefully downgrading for standard 2024-11-05 clients (due to rust-mcp-sdk's strict `cmp`).
        protocol_version: "2025-11-25".to_string(),
        instructions: None,
        meta: None,
    };

    let transport = StdioTransport::new(TransportOptions::default());
    let transport = match transport {
        Ok(t) => t,
        Err(e) => panic!("Transport error: {}", e),
    };

    let handler = ScreenshotHandler::default().to_mcp_server_handler();

    let options = McpServerOptions {
        server_details: server_info,
        transport,
        handler,
        task_store: None,
        client_task_store: None,
        message_observer: None,
    };

    let server = server_runtime::create_server(options);
    server.start().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
