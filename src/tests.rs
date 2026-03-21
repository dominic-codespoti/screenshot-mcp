use super::*;

#[tokio::test]
async fn test_tool_definitions() {
    let list_tool = ListScreenshotTargetsTool::tool();
    assert_eq!(list_tool.name, "list_screenshot_targets");
    
    let take_tool = TakeScreenshotTool::tool();
    assert_eq!(take_tool.name, "take_screenshot");
}

#[tokio::test]
async fn test_valid_json() {
    let tool_args = serde_json::json!({
        "target_type": "pid",
        "target_id": "1234"
    });
    let parsed: TakeScreenshotTool = serde_json::from_value(tool_args).unwrap();
    assert_eq!(parsed.target_type, "pid");
    assert_eq!(parsed.target_id.unwrap(), "1234");
}

#[tokio::test]
async fn test_monitor_json() {
    let tool_args = serde_json::json!({
        "target_type": "primary_monitor"
    });
    let parsed: TakeScreenshotTool = serde_json::from_value(tool_args).unwrap();
    assert_eq!(parsed.target_type, "primary_monitor");
    assert_eq!(parsed.target_id, None);
}
