use serde_json::Value;

use crate::{ContentBlock, Message, MessageRole, TauAiError, ToolCall};

pub fn promote_assistant_textual_tool_calls(message: Message) -> Result<Message, TauAiError> {
    if message.role != MessageRole::Assistant || !message.tool_calls().is_empty() {
        return Ok(message);
    }

    let text = message.text_content();
    let Some(tool_calls) = extract_textual_tool_calls(&text)? else {
        return Ok(message);
    };

    Ok(Message::assistant_blocks(
        tool_calls.into_iter().map(ContentBlock::tool_call).collect(),
    ))
}

fn extract_textual_tool_calls(payload: &str) -> Result<Option<Vec<ToolCall>>, TauAiError> {
    let trimmed = payload.trim();
    if !looks_like_tool_call_payload(trimmed) {
        return Ok(None);
    }

    parse_tool_calls_payload(trimmed).ok_or_else(|| {
        TauAiError::InvalidResponse(
            "textual tool-call promotion failed: assistant payload looked like tool-call JSON but could not be parsed".to_string(),
        )
    })
    .map(Some)
}

fn looks_like_tool_call_payload(payload: &str) -> bool {
    payload.starts_with('{')
        && (payload.contains("\"tool_calls\"")
            || payload.contains("\"tool_call\"")
            || payload.contains("\"assistant_text\""))
}

fn parse_tool_calls_payload(payload: &str) -> Option<Vec<ToolCall>> {
    let parsed = serde_json::from_str::<Value>(payload).ok()?;
    parse_tool_calls_value(&parsed)
}

fn parse_tool_calls_value(value: &Value) -> Option<Vec<ToolCall>> {
    if let Some(call) = parse_tool_call_entry(value) {
        return Some(vec![call]);
    }

    let object = value.as_object()?;
    if let Some(call) = object.get("tool_call").and_then(parse_tool_call_entry) {
        return Some(vec![call]);
    }
    if let Some(calls) = object.get("tool_calls").and_then(Value::as_array) {
        let parsed = calls
            .iter()
            .filter_map(parse_tool_call_entry)
            .collect::<Vec<_>>();
        if !parsed.is_empty() {
            return Some(parsed);
        }
    }

    let nested = object.get("assistant_text")?.as_str()?;
    parse_tool_calls_payload(nested)
}

fn parse_tool_call_entry(value: &Value) -> Option<ToolCall> {
    let object = value.as_object()?;
    if let (Some(id), Some(name)) = (
        object.get("id").and_then(Value::as_str),
        object.get("name").and_then(Value::as_str),
    ) {
        return Some(ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            arguments: normalize_arguments(object.get("arguments").cloned()),
        });
    }

    let function = object.get("function")?.as_object()?;
    let name = function.get("name")?.as_str()?;
    Some(ToolCall {
        id: object
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("call_{name}")),
        name: name.to_string(),
        arguments: normalize_arguments(function.get("arguments").cloned()),
    })
}

fn normalize_arguments(arguments: Option<Value>) -> Value {
    match arguments.unwrap_or(Value::Object(Default::default())) {
        Value::String(serialized) => {
            serde_json::from_str(&serialized).unwrap_or(Value::String(serialized))
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::promote_assistant_textual_tool_calls;
    use crate::Message;
    use serde_json::json;

    #[test]
    fn promotes_wrapped_assistant_text_payload_into_tool_calls() {
        let message = Message::assistant_text(
            "{\"assistant_text\":\"{\\\"tool_calls\\\":[{\\\"id\\\":\\\"call_1\\\",\\\"name\\\":\\\"bash\\\",\\\"arguments\\\":{\\\"command\\\":\\\"pwd\\\"}}]}\"}",
        );

        let promoted = promote_assistant_textual_tool_calls(message).expect("promotion");
        let calls = promoted.tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "bash");
        assert_eq!(calls[0].arguments, json!({"command":"pwd"}));
    }

    #[test]
    fn rejects_malformed_tool_call_candidate() {
        let message =
            Message::assistant_text("{\"tool_calls\":[{\"id\":\"call_1\",\"name\":\"bash\"");
        let error = promote_assistant_textual_tool_calls(message).expect_err("must fail");
        assert!(error
            .to_string()
            .contains("textual tool-call promotion failed"));
    }
}
