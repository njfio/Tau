//! Gateway stream response handler for `/v1/responses` stream mode.

use super::*;

pub(super) async fn stream_openresponses(
    state: Arc<GatewayOpenResponsesServerState>,
    request: OpenResponsesRequest,
) -> Response {
    let (tx, rx) = mpsc::unbounded_channel::<SseFrame>();
    tokio::spawn(async move {
        match execute_openresponses_request(state, request, Some(tx.clone())).await {
            Ok(result) => {
                let response = result.response;
                let _ = tx.send(SseFrame::Json {
                    event: "response.output_text.done",
                    payload: response_output_text_done_payload(
                        &response.id,
                        response.output_text.clone(),
                    ),
                });
                let _ = tx.send(SseFrame::Json {
                    event: "response.completed",
                    payload: response_completed_payload(response),
                });
                let _ = tx.send(SseFrame::Done);
            }
            Err(error) => {
                let _ = tx.send(SseFrame::Json {
                    event: "response.failed",
                    payload: response_failed_payload(&error),
                });
                let _ = tx.send(SseFrame::Done);
            }
        }
    });

    let stream =
        UnboundedReceiverStream::new(rx).map(|frame| Ok::<Event, Infallible>(frame.into_event()));
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}
