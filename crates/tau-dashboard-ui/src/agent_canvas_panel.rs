use leptos::prelude::*;

pub(crate) fn render_tau_ops_chat_agent_canvas(
    status: String,
    loaded: String,
    artifact_path: String,
    srcdoc: String,
    srcdoc_bytes: String,
    placeholder_hidden: String,
    loaded_bool: bool,
) -> impl IntoView {
    view! {
        <section
            id="tau-ops-chat-agent-canvas"
            data-agent-canvas="true"
            data-preview-status=status
            data-preview-loaded=loaded
            data-artifact-path=artifact_path
            data-srcdoc-bytes=srcdoc_bytes
        >
            <h3>Agent Canvas</h3>
            <canvas
                id="tau-ops-chat-agent-canvas-surface"
                data-agent-canvas-surface="true"
                width="720"
                height="405"
                aria-hidden=placeholder_hidden
            ></canvas>
            {if loaded_bool {
                leptos::either::Either::Left(view! {
                    <iframe
                        id="tau-ops-chat-agent-preview-frame"
                        data-agent-html-preview="true"
                        sandbox="allow-scripts"
                        title="Agent HTML preview"
                        srcdoc=srcdoc
                    ></iframe>
                })
            } else {
                leptos::either::Either::Right(view! {
                    <span
                        id="tau-ops-chat-agent-preview-empty"
                        data-agent-html-preview="false"
                        hidden
                    ></span>
                })
            }}
        </section>
    }
}
