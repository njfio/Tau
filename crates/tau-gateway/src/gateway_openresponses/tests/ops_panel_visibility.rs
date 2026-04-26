use super::*;

#[tokio::test]
async fn functional_spec_2858_c01_c02_ops_shell_panels_expose_visibility_state_markers_on_primary_routes(
) {
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let chat_response = client
        .get(format!(
            "http://{addr}/ops/chat?theme=light&sidebar=collapsed&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops chat shell request");
    assert_eq!(chat_response.status(), StatusCode::OK);
    let chat_body = chat_response
        .text()
        .await
        .expect("read ops chat shell body");
    assert!(chat_body.contains(
        "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"false\" data-active-session-key=\"chat-c01\" data-panel-visible=\"true\""
    ));

    let sessions_response = client
        .get(format!(
            "http://{addr}/ops/sessions?theme=dark&sidebar=expanded&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops sessions shell request");
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_body = sessions_response
        .text()
        .await
        .expect("read ops sessions shell body");
    assert!(sessions_body.contains(
        "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2858_c03_c04_c05_ops_shell_panel_visibility_state_combinations_by_route()
{
    let temp = tempdir().expect("tempdir");
    write_dashboard_runtime_fixture(temp.path());
    write_training_runtime_fixture(temp.path(), 0);
    let state = test_state(temp.path(), 4_096, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let chat_response = client
        .get(format!(
            "http://{addr}/ops/chat?theme=light&sidebar=collapsed&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops chat shell request");
    assert_eq!(chat_response.status(), StatusCode::OK);
    let chat_body = chat_response
        .text()
        .await
        .expect("read ops chat shell body");
    assert!(chat_body.contains(
        "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"false\" data-active-session-key=\"chat-c01\" data-panel-visible=\"true\""
    ));
    assert!(chat_body.contains(
        "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));

    let sessions_response = client
        .get(format!(
            "http://{addr}/ops/sessions?theme=dark&sidebar=expanded&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops sessions shell request");
    assert_eq!(sessions_response.status(), StatusCode::OK);
    let sessions_body = sessions_response
        .text()
        .await
        .expect("read ops sessions shell body");
    assert!(sessions_body.contains(
        "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-c01\" data-panel-visible=\"false\""
    ));
    assert!(sessions_body.contains(
        "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));

    let ops_response = client
        .get(format!(
            "http://{addr}/ops?theme=dark&sidebar=expanded&session=chat-c01"
        ))
        .send()
        .await
        .expect("ops shell request");
    assert_eq!(ops_response.status(), StatusCode::OK);
    let ops_body = ops_response.text().await.expect("read ops shell body");
    assert!(ops_body.contains(
        "id=\"tau-ops-chat-panel\" data-route=\"/ops/chat\" aria-hidden=\"true\" data-active-session-key=\"chat-c01\" data-panel-visible=\"false\""
    ));
    assert!(ops_body.contains(
        "id=\"tau-ops-sessions-panel\" data-route=\"/ops/sessions\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));

    handle.abort();
}
