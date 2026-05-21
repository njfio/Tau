use std::path::PathBuf;

use serde_json::Value;

const OPS_CHAT_AGENT_CANVAS_PREVIEW_MAX_BYTES: u64 = 512 * 1024;

const OPS_CHAT_AGENT_CANVAS_FRAME_SCRIPT: &str = r#"<script data-agent-canvas-bridge="true">
(function () {
    var channel = "tau-agent-canvas-v2";
    var consoleEvents = [];

    function safeString(value) {
        try {
            if (typeof value === "string") {
                return value;
            }
            return JSON.stringify(value);
        } catch (_) {
            return String(value);
        }
    }

    function recordConsole(level, args) {
        consoleEvents.push({
            level: level,
            message: Array.prototype.slice.call(args).map(safeString).join(" "),
            timestamp: Date.now()
        });
        if (consoleEvents.length > 12) {
            consoleEvents.shift();
        }
    }

    ["log", "warn", "error"].forEach(function (level) {
        var original = console[level];
        console[level] = function () {
            recordConsole(level, arguments);
            if (typeof original === "function") {
                original.apply(console, arguments);
            }
        };
    });

    window.addEventListener("error", function (event) {
        recordConsole("error", [event.message || "script error"]);
        postDiagnostics("error");
    });

    window.addEventListener("unhandledrejection", function (event) {
        recordConsole("error", [event.reason || "unhandled rejection"]);
        postDiagnostics("unhandledrejection");
    });

    function canvasSample(canvas) {
        var width = Number(canvas.width || canvas.clientWidth || 0);
        var height = Number(canvas.height || canvas.clientHeight || 0);
        var sample = {
            id: canvas.id || "",
            width: width,
            height: height,
            status: "empty",
            pixels: []
        };
        if (!width || !height) {
            return sample;
        }
        try {
            var context = canvas.getContext("2d");
            var points = [
                [Math.max(0, Math.floor(width / 2)), Math.max(0, Math.floor(height / 2))],
                [0, 0],
                [Math.max(0, width - 1), Math.max(0, height - 1)]
            ];
            sample.pixels = points.map(function (point) {
                var data = context.getImageData(point[0], point[1], 1, 1).data;
                return {
                    x: point[0],
                    y: point[1],
                    rgba: [data[0], data[1], data[2], data[3]]
                };
            });
            sample.status = "sampled";
        } catch (error) {
            sample.status = "blocked";
            sample.error = safeString(error && error.message ? error.message : error);
        }
        return sample;
    }

    function postDiagnostics(reason) {
        var canvases = Array.prototype.slice.call(document.querySelectorAll("canvas"))
            .slice(0, 6)
            .map(canvasSample);
        parent.postMessage({
            channel: channel,
            type: "diagnostics",
            reason: reason,
            dom_node_count: document.querySelectorAll("*").length,
            canvas_count: canvases.length,
            console_events: consoleEvents.slice(-12),
            canvas_samples: canvases
        }, "*");
    }

    function dispatchClick(command) {
        var x = Number(command.x || 0);
        var y = Number(command.y || 0);
        var target = document.elementFromPoint(x, y);
        if (!target) {
            recordConsole("warn", ["click target missing", x, y]);
            return;
        }
        target.dispatchEvent(new MouseEvent("click", {
            bubbles: true,
            cancelable: true,
            clientX: x,
            clientY: y,
            view: window
        }));
    }

    function dispatchType(command) {
        var text = safeString(command.text || "");
        var target = document.activeElement;
        if (!target || target === document.body) {
            target = document.querySelector("input, textarea, [contenteditable=\"true\"]") || document.body;
        }
        if ("value" in target) {
            target.value = String(target.value || "") + text;
            target.dispatchEvent(new InputEvent("input", { bubbles: true, data: text }));
        } else if (target.isContentEditable) {
            target.textContent = String(target.textContent || "") + text;
            target.dispatchEvent(new InputEvent("input", { bubbles: true, data: text }));
        } else {
            target.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: text }));
        }
    }

    window.addEventListener("message", function (event) {
        var command = event.data || {};
        if (command.channel !== channel || command.type !== "command") {
            return;
        }
        if (command.action === "click") {
            dispatchClick(command);
        } else if (command.action === "type") {
            dispatchType(command);
        }
        postDiagnostics(command.action || "command");
    });

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", function () {
            postDiagnostics("domcontentloaded");
        });
    } else {
        postDiagnostics("ready");
    }
    window.addEventListener("load", function () {
        postDiagnostics("load");
    });
})();
</script>"#;

const OPS_CHAT_AGENT_CANVAS_PARENT_STYLE: &str = r#"<style id="tau-ops-chat-agent-canvas-v2-style">
#tau-ops-chat-agent-canvas-controls {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
    align-items: end;
}
#tau-ops-chat-agent-canvas-controls label {
    display: grid;
    gap: 3px;
    color: #9eb8c4;
    font-size: .72rem;
}
#tau-ops-chat-agent-canvas-controls input {
    min-width: 0;
}
#tau-ops-chat-agent-canvas-diagnostics,
#tau-ops-chat-agent-canvas-artifacts {
    display: grid;
    gap: 6px;
    margin: 0;
    padding: 8px;
    border: 1px solid #203847;
    border-radius: 6px;
    background: #041017;
    color: #c9dce5;
    font-size: .74rem;
}
#tau-ops-chat-agent-canvas-diagnostics dl {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
    margin: 0;
}
#tau-ops-chat-agent-canvas-diagnostics dt {
    color: #7f9cac;
    font-size: .68rem;
}
#tau-ops-chat-agent-canvas-diagnostics dd {
    margin: 0;
    color: #edf8fb;
    overflow-wrap: anywhere;
}
#tau-ops-chat-agent-canvas-artifacts {
    list-style-position: inside;
}
#tau-ops-chat-agent-canvas-artifacts li {
    overflow-wrap: anywhere;
}
#tau-ops-chat-agent-canvas-console,
#tau-ops-chat-agent-canvas-pixels {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
}
</style>"#;

const OPS_CHAT_AGENT_CANVAS_PARENT_SCRIPT: &str = r#"<script id="tau-ops-chat-agent-canvas-runtime" data-agent-canvas-runtime="postmessage-v2">
(function () {
    var section = document.getElementById("tau-ops-chat-agent-canvas");
    var frame = document.getElementById("tau-ops-chat-agent-preview-frame");
    if (!section || section.getAttribute("data-agent-canvas-runtime-bound") === "true") {
        return;
    }
    section.setAttribute("data-agent-canvas-runtime-bound", "true");

    var channel = "tau-agent-canvas-v2";
    var runtimeStatus = document.getElementById("tau-ops-chat-agent-canvas-runtime-status");
    var domCount = document.getElementById("tau-ops-chat-agent-canvas-dom-count");
    var canvasCount = document.getElementById("tau-ops-chat-agent-canvas-count");
    var consoleCount = document.getElementById("tau-ops-chat-agent-canvas-console-count");
    var consoleList = document.getElementById("tau-ops-chat-agent-canvas-console");
    var pixelList = document.getElementById("tau-ops-chat-agent-canvas-pixels");

    function setText(element, value) {
        if (element) {
            element.textContent = String(value);
        }
    }

    function sendCommand(action) {
        if (!frame || !frame.contentWindow) {
            section.setAttribute("data-preview-runtime-status", "missing-frame");
            setText(runtimeStatus, "missing-frame");
            return;
        }
        var xInput = document.getElementById("tau-ops-chat-agent-canvas-click-x");
        var yInput = document.getElementById("tau-ops-chat-agent-canvas-click-y");
        var textInput = document.getElementById("tau-ops-chat-agent-canvas-type-text");
        frame.contentWindow.postMessage({
            channel: channel,
            type: "command",
            action: action,
            x: xInput ? Number(xInput.value || 0) : 0,
            y: yInput ? Number(yInput.value || 0) : 0,
            text: textInput ? textInput.value || "" : ""
        }, "*");
    }

    function renderList(list, rows, format) {
        if (!list) {
            return;
        }
        list.textContent = "";
        rows.slice(-8).forEach(function (row) {
            var item = document.createElement("li");
            item.textContent = format(row);
            list.appendChild(item);
        });
    }

    window.addEventListener("message", function (event) {
        var data = event.data || {};
        if (!frame || event.source !== frame.contentWindow || data.channel !== channel || data.type !== "diagnostics") {
            return;
        }
        var consoleEvents = Array.isArray(data.console_events) ? data.console_events : [];
        var pixelSamples = Array.isArray(data.canvas_samples) ? data.canvas_samples : [];
        var errorCount = consoleEvents.filter(function (event) {
            return event && event.level === "error";
        }).length;
        section.setAttribute("data-preview-runtime-status", "connected");
        section.setAttribute("data-dom-node-count", String(data.dom_node_count || 0));
        section.setAttribute("data-canvas-count", String(data.canvas_count || 0));
        section.setAttribute("data-console-error-count", String(errorCount));
        section.setAttribute("data-pixel-sample-count", String(pixelSamples.length));
        setText(runtimeStatus, "connected");
        setText(domCount, data.dom_node_count || 0);
        setText(canvasCount, data.canvas_count || 0);
        setText(consoleCount, errorCount);
        renderList(consoleList, consoleEvents, function (event) {
            return String(event.level || "log") + ": " + String(event.message || "");
        });
        renderList(pixelList, pixelSamples, function (sample) {
            return "canvas " + String(sample.id || "(anonymous)") + " " + String(sample.status || "unknown") + " " + String(sample.width || 0) + "x" + String(sample.height || 0);
        });
    });

    var probe = document.getElementById("tau-ops-chat-agent-canvas-probe");
    var click = document.getElementById("tau-ops-chat-agent-canvas-click");
    var type = document.getElementById("tau-ops-chat-agent-canvas-type");
    if (probe) {
        probe.addEventListener("click", function () { sendCommand("snapshot"); });
    }
    if (click) {
        click.addEventListener("click", function () { sendCommand("click"); });
    }
    if (type) {
        type.addEventListener("click", function () { sendCommand("type"); });
    }
    if (frame && frame.contentWindow) {
        setTimeout(function () { sendCommand("snapshot"); }, 50);
    }
})();
</script>"#;

#[derive(Debug, Clone)]
pub(super) struct OpsChatAgentCanvasPreview {
    pub(super) status: String,
    pub(super) artifact_path: String,
    pub(super) srcdoc: String,
    pub(super) srcdoc_bytes: usize,
}

impl Default for OpsChatAgentCanvasPreview {
    fn default() -> Self {
        Self {
            status: "empty".to_string(),
            artifact_path: String::new(),
            srcdoc: String::new(),
            srcdoc_bytes: 0,
        }
    }
}

pub(super) fn push_ops_chat_agent_canvas_preview(
    previews: &mut Vec<OpsChatAgentCanvasPreview>,
    content: &str,
) {
    let Some(path) = extract_ops_chat_html_artifact_path(content) else {
        return;
    };
    let preview = load_ops_chat_agent_canvas_preview(path.as_str());
    if let Some(existing) = previews
        .iter_mut()
        .find(|existing| existing.artifact_path == preview.artifact_path)
    {
        *existing = preview;
    } else {
        previews.push(preview);
    }
}

pub(super) fn upgrade_ops_chat_agent_canvas_html(
    html: String,
    previews: &[OpsChatAgentCanvasPreview],
) -> String {
    let with_runtime_attrs = add_agent_canvas_runtime_attrs(html, previews.len());
    inject_agent_canvas_runtime_panel(with_runtime_attrs, previews)
}

fn add_agent_canvas_runtime_attrs(html: String, artifact_count: usize) -> String {
    let marker = r#"id="tau-ops-chat-agent-canvas" data-agent-canvas="true""#;
    let replacement = format!(
        r#"{marker} data-artifact-count="{artifact_count}" data-preview-runtime-status="pending" data-dom-node-count="0" data-canvas-count="0" data-console-error-count="0" data-pixel-sample-count="0" data-interaction-mode="postmessage""#
    );
    html.replacen(marker, replacement.as_str(), 1)
}

fn inject_agent_canvas_runtime_panel(
    mut html: String,
    previews: &[OpsChatAgentCanvasPreview],
) -> String {
    let Some(canvas_id_index) = html.find(r#"id="tau-ops-chat-agent-canvas""#) else {
        return html;
    };
    let Some(section_start_index) = html[..canvas_id_index].rfind("<section") else {
        return html;
    };
    let Some(section_end_offset) = html[canvas_id_index..].find("</section>") else {
        return html;
    };
    let section_end_index = canvas_id_index + section_end_offset;
    let panel_html = render_agent_canvas_runtime_panel(previews);
    if html[section_start_index..section_end_index].contains("tau-ops-chat-agent-canvas-runtime") {
        return html;
    }
    html.insert_str(section_end_index, panel_html.as_str());
    html
}

fn render_agent_canvas_runtime_panel(previews: &[OpsChatAgentCanvasPreview]) -> String {
    let mut html = String::new();
    html.push_str(OPS_CHAT_AGENT_CANVAS_PARENT_STYLE);
    html.push_str(
        r#"<form id="tau-ops-chat-agent-canvas-controls" data-agent-canvas-controls="postmessage"><label for="tau-ops-chat-agent-canvas-click-x">X<input id="tau-ops-chat-agent-canvas-click-x" type="number" value="40" min="0"/></label><label for="tau-ops-chat-agent-canvas-click-y">Y<input id="tau-ops-chat-agent-canvas-click-y" type="number" value="40" min="0"/></label><label for="tau-ops-chat-agent-canvas-type-text">Text<input id="tau-ops-chat-agent-canvas-type-text" type="text" value="" autocomplete="off"/></label><button id="tau-ops-chat-agent-canvas-probe" type="button" data-agent-canvas-tool="snapshot">Probe</button><button id="tau-ops-chat-agent-canvas-click" type="button" data-agent-canvas-tool="click">Click</button><button id="tau-ops-chat-agent-canvas-type" type="button" data-agent-canvas-tool="type">Type</button></form><section id="tau-ops-chat-agent-canvas-diagnostics" data-agent-canvas-diagnostics="true"><dl><div><dt>Runtime</dt><dd id="tau-ops-chat-agent-canvas-runtime-status">pending</dd></div><div><dt>DOM</dt><dd id="tau-ops-chat-agent-canvas-dom-count">0</dd></div><div><dt>Canvas</dt><dd id="tau-ops-chat-agent-canvas-count">0</dd></div><div><dt>Console</dt><dd id="tau-ops-chat-agent-canvas-console-count">0</dd></div></dl><ul id="tau-ops-chat-agent-canvas-console" data-agent-canvas-console-events="true"></ul><ul id="tau-ops-chat-agent-canvas-pixels" data-agent-canvas-pixel-samples="true"></ul></section>"#,
    );
    html.push_str(
        format!(
            r#"<ol id="tau-ops-chat-agent-canvas-artifacts" data-agent-canvas-artifact-history="true" data-artifact-count="{}">"#,
            previews.len()
        )
        .as_str(),
    );
    for (index, preview) in previews.iter().enumerate() {
        let artifact_index = index + 1;
        let artifact_id = format!("tau-ops-chat-agent-canvas-artifact-{artifact_index}");
        let escaped_artifact_id = escape_ops_chat_agent_canvas_html(artifact_id.as_str());
        let escaped_status = escape_ops_chat_agent_canvas_html(preview.status.as_str());
        let escaped_path = escape_ops_chat_agent_canvas_html(preview.artifact_path.as_str());
        html.push_str(
            format!(
                r#"<li id="{escaped_artifact_id}" data-artifact-index="{artifact_index}" data-preview-status="{escaped_status}" data-artifact-path="{escaped_path}" data-srcdoc-bytes="{}"><code>{escaped_path}</code></li>"#,
                preview.srcdoc_bytes
            )
            .as_str(),
        );
    }
    html.push_str("</ol>");
    html.push_str(OPS_CHAT_AGENT_CANVAS_PARENT_SCRIPT);
    html
}

fn escape_ops_chat_agent_canvas_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn extract_ops_chat_html_artifact_path(content: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(content).ok()?;
    find_ops_chat_html_path_in_value(&value).map(str::to_string)
}

fn find_ops_chat_html_path_in_value(value: &Value) -> Option<&str> {
    match value {
        Value::Object(map) => {
            for key in ["path", "file", "artifact_path", "href"] {
                if let Some(path) = map.get(key).and_then(Value::as_str) {
                    if is_ops_chat_html_path(path) {
                        return Some(path);
                    }
                }
            }
            map.values().find_map(find_ops_chat_html_path_in_value)
        }
        Value::Array(values) => values.iter().find_map(find_ops_chat_html_path_in_value),
        Value::String(path) if is_ops_chat_html_path(path) => Some(path.as_str()),
        _ => None,
    }
}

fn is_ops_chat_html_path(path: &str) -> bool {
    let normalized = path.trim().to_ascii_lowercase();
    normalized.ends_with(".html") || normalized.ends_with(".htm")
}

fn load_ops_chat_agent_canvas_preview(path: &str) -> OpsChatAgentCanvasPreview {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return OpsChatAgentCanvasPreview::default();
    }

    let mut preview = OpsChatAgentCanvasPreview {
        status: "missing".to_string(),
        artifact_path: trimmed.to_string(),
        srcdoc: String::new(),
        srcdoc_bytes: 0,
    };
    let Some(resolved) = resolve_ops_chat_agent_canvas_artifact_path(trimmed) else {
        preview.status = "unsafe_path".to_string();
        return preview;
    };
    let Ok(metadata) = std::fs::metadata(&resolved) else {
        preview.status = "missing".to_string();
        return preview;
    };
    if metadata.len() > OPS_CHAT_AGENT_CANVAS_PREVIEW_MAX_BYTES {
        preview.status = "too_large".to_string();
        return preview;
    }
    match std::fs::read_to_string(&resolved) {
        Ok(srcdoc) => {
            preview.status = "loaded".to_string();
            preview.artifact_path = resolved.display().to_string();
            preview.srcdoc_bytes = srcdoc.len();
            preview.srcdoc = instrument_ops_chat_agent_canvas_srcdoc(srcdoc.as_str());
            preview
        }
        Err(_) => {
            preview.status = "read_error".to_string();
            preview
        }
    }
}

fn instrument_ops_chat_agent_canvas_srcdoc(srcdoc: &str) -> String {
    let mut output = String::with_capacity(srcdoc.len() + OPS_CHAT_AGENT_CANVAS_FRAME_SCRIPT.len());
    let lower = srcdoc.to_ascii_lowercase();
    if let Some(index) = lower.rfind("</body>") {
        output.push_str(&srcdoc[..index]);
        output.push_str(OPS_CHAT_AGENT_CANVAS_FRAME_SCRIPT);
        output.push_str(&srcdoc[index..]);
    } else {
        output.push_str(srcdoc);
        output.push('\n');
        output.push_str(OPS_CHAT_AGENT_CANVAS_FRAME_SCRIPT);
    }
    output
}

fn resolve_ops_chat_agent_canvas_artifact_path(path: &str) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?.canonicalize().ok()?;
    let raw = PathBuf::from(path);
    let candidate = if raw.is_absolute() {
        raw
    } else {
        cwd.join(raw)
    };
    let canonical = candidate.canonicalize().ok()?;
    if canonical.starts_with(&cwd) && is_ops_chat_html_path(canonical.to_string_lossy().as_ref()) {
        Some(canonical)
    } else {
        None
    }
}
