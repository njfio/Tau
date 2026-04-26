use super::*;

#[tokio::test]
async fn integration_spec_2905_c01_c02_c03_ops_memory_route_renders_relevant_search_results_and_empty_state(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-search";
    for index in 1..=6 {
        let memory_id = format!("mem-match-{index}");
        let entry_endpoint =
            expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, &memory_id);
        let create_match = client
            .put(format!("http://{addr}{entry_endpoint}"))
            .bearer_auth("secret")
            .json(&json!({
                "summary": "ArcSwap",
                "memory_type": "fact",
                "workspace_id": "workspace-a",
                "channel_id": "gateway",
                "actor_id": "operator",
                "policy_gate": MEMORY_WRITE_POLICY_GATE
            }))
            .send()
            .await
            .expect("create matching memory entry");
        assert_eq!(create_match.status(), StatusCode::CREATED);
    }

    let cross_workspace_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-cross-workspace",
    );
    let create_cross_workspace = client
        .put(format!("http://{addr}{cross_workspace_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "ArcSwap",
            "memory_type": "fact",
            "workspace_id": "workspace-b",
            "channel_id": "gateway",
            "actor_id": "operator",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create cross-workspace memory entry");
    assert_eq!(create_cross_workspace.status(), StatusCode::CREATED);

    let query_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=ArcSwap&workspace_id=workspace-a&limit=25"
        ))
        .send()
        .await
        .expect("ops memory search request");
    assert_eq!(query_response.status(), StatusCode::OK);
    let query_body = query_response.text().await.expect("read ops memory body");
    assert!(query_body.contains(
        "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"false\" data-panel-visible=\"true\" data-query=\"ArcSwap\""
    ));
    assert!(query_body.contains("data-result-count=\"6\""));
    assert!(query_body
        .contains("id=\"tau-ops-memory-search-form\" action=\"/ops/memory\" method=\"get\""));
    assert!(query_body
        .contains("id=\"tau-ops-memory-query\" type=\"search\" name=\"query\" value=\"ArcSwap\""));
    assert!(query_body.contains(
        "id=\"tau-ops-memory-result-row-0\" data-memory-id=\"mem-match-1\" data-memory-type=\"fact\""
    ));
    assert!(query_body.contains("id=\"tau-ops-memory-result-row-5\""));
    assert!(query_body.contains("ArcSwap"));
    assert!(!query_body.contains("mem-cross-workspace"));

    let empty_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=NoHitTerm"
        ))
        .send()
        .await
        .expect("ops memory no-hit request");
    assert_eq!(empty_response.status(), StatusCode::OK);
    let empty_body = empty_response
        .text()
        .await
        .expect("read ops memory empty body");
    assert!(empty_body.contains(
        "id=\"tau-ops-memory-panel\" data-route=\"/ops/memory\" aria-hidden=\"false\" data-panel-visible=\"true\" data-query=\"NoHitTerm\" data-result-count=\"0\""
    ));
    assert!(empty_body.contains("id=\"tau-ops-memory-results\" data-result-count=\"0\""));
    assert!(empty_body.contains("id=\"tau-ops-memory-empty-state\" data-empty-state=\"true\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2909_c01_c02_c03_ops_memory_scope_filters_narrow_results() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-scope-filter";
    let fixtures = [
        (
            "mem-scope-target",
            "workspace-a",
            "channel-alpha",
            "operator",
        ),
        (
            "mem-scope-workspace-miss",
            "workspace-b",
            "channel-alpha",
            "operator",
        ),
        (
            "mem-scope-channel-miss",
            "workspace-a",
            "channel-beta",
            "operator",
        ),
        (
            "mem-scope-actor-miss",
            "workspace-a",
            "channel-alpha",
            "observer",
        ),
    ];

    for (memory_id, workspace_id, channel_id, actor_id) in fixtures {
        let entry_endpoint =
            expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, memory_id);
        let create = client
            .put(format!("http://{addr}{entry_endpoint}"))
            .bearer_auth("secret")
            .json(&json!({
                "summary": "ScopeToken",
                "memory_type": "fact",
                "workspace_id": workspace_id,
                "channel_id": channel_id,
                "actor_id": actor_id,
                "policy_gate": MEMORY_WRITE_POLICY_GATE
            }))
            .send()
            .await
            .expect("create scope fixture entry");
        assert_eq!(create.status(), StatusCode::CREATED);
    }

    let scoped_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=ScopeToken&workspace_id=workspace-a&channel_id=channel-alpha&actor_id=operator&limit=25"
        ))
        .send()
        .await
        .expect("ops memory scoped request");
    assert_eq!(scoped_response.status(), StatusCode::OK);
    let scoped_body = scoped_response
        .text()
        .await
        .expect("read scoped response body");
    assert!(scoped_body.contains(
        "id=\"tau-ops-memory-workspace-filter\" type=\"text\" name=\"workspace_id\" value=\"workspace-a\""
    ));
    assert!(scoped_body.contains(
        "id=\"tau-ops-memory-channel-filter\" type=\"text\" name=\"channel_id\" value=\"channel-alpha\""
    ));
    assert!(scoped_body.contains(
        "id=\"tau-ops-memory-actor-filter\" type=\"text\" name=\"actor_id\" value=\"operator\""
    ));
    assert!(scoped_body.contains("id=\"tau-ops-memory-results\" data-result-count=\"1\""));
    assert!(scoped_body.contains(
        "id=\"tau-ops-memory-result-row-0\" data-memory-id=\"mem-scope-target\" data-memory-type=\"fact\""
    ));
    assert!(!scoped_body.contains("mem-scope-workspace-miss"));
    assert!(!scoped_body.contains("mem-scope-channel-miss"));
    assert!(!scoped_body.contains("mem-scope-actor-miss"));

    let no_match_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=ScopeToken&workspace_id=workspace-a&channel_id=channel-alpha&actor_id=no-match"
        ))
        .send()
        .await
        .expect("ops memory scoped no-match request");
    assert_eq!(no_match_response.status(), StatusCode::OK);
    let no_match_body = no_match_response
        .text()
        .await
        .expect("read scoped no-match body");
    assert!(no_match_body.contains("id=\"tau-ops-memory-results\" data-result-count=\"0\""));
    assert!(no_match_body.contains("id=\"tau-ops-memory-empty-state\" data-empty-state=\"true\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2913_c01_c02_c03_ops_memory_type_filter_narrows_results() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-type-filter";
    let fixtures = [
        ("mem-type-fact", "fact"),
        ("mem-type-goal", "goal"),
        ("mem-type-decision", "decision"),
    ];

    for (memory_id, memory_type) in fixtures {
        let entry_endpoint =
            expand_memory_entry_template(GATEWAY_MEMORY_ENTRY_ENDPOINT, session_key, memory_id);
        let create = client
            .put(format!("http://{addr}{entry_endpoint}"))
            .bearer_auth("secret")
            .json(&json!({
                "summary": "TypeToken",
                "memory_type": memory_type,
                "workspace_id": "workspace-a",
                "channel_id": "channel-alpha",
                "actor_id": "operator",
                "policy_gate": MEMORY_WRITE_POLICY_GATE
            }))
            .send()
            .await
            .expect("create type fixture entry");
        assert_eq!(create.status(), StatusCode::CREATED);
    }

    let filtered_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=TypeToken&workspace_id=workspace-a&channel_id=channel-alpha&actor_id=operator&memory_type=fact&limit=25"
        ))
        .send()
        .await
        .expect("ops memory type-filter request");
    assert_eq!(filtered_response.status(), StatusCode::OK);
    let filtered_body = filtered_response
        .text()
        .await
        .expect("read type-filter response body");
    assert!(filtered_body.contains(
        "id=\"tau-ops-memory-type-filter\" type=\"text\" name=\"memory_type\" value=\"fact\""
    ));
    assert!(filtered_body.contains("id=\"tau-ops-memory-results\" data-result-count=\"1\""));
    assert!(filtered_body.contains(
        "id=\"tau-ops-memory-result-row-0\" data-memory-id=\"mem-type-fact\" data-memory-type=\"fact\""
    ));
    assert!(!filtered_body.contains("mem-type-goal"));
    assert!(!filtered_body.contains("mem-type-decision"));

    let no_match_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=TypeToken&workspace_id=workspace-a&channel_id=channel-alpha&actor_id=operator&memory_type=identity"
        ))
        .send()
        .await
        .expect("ops memory type-filter no-match request");
    assert_eq!(no_match_response.status(), StatusCode::OK);
    let no_match_body = no_match_response
        .text()
        .await
        .expect("read type-filter no-match body");
    assert!(no_match_body.contains("id=\"tau-ops-memory-results\" data-result-count=\"0\""));
    assert!(no_match_body.contains("id=\"tau-ops-memory-empty-state\" data-empty-state=\"true\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2917_c02_c03_ops_memory_create_submission_persists_entry_and_sets_status_markers(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build client");

    let session_key = "ops-memory-create";
    let related_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-create-related",
    );
    let related_create = client
        .put(format!("http://{addr}{related_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "CreateToken relation target",
            "memory_type": "fact",
            "workspace_id": "workspace-create",
            "channel_id": "channel-create",
            "actor_id": "operator",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create related memory entry");
    assert_eq!(related_create.status(), StatusCode::CREATED);

    let create_response = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("entry_id", "mem-create-1"),
            ("summary", "CreateToken summary"),
            ("tags", "alpha,beta"),
            ("facts", "fact-one|fact-two"),
            ("source_event_key", "evt-create-1"),
            ("workspace_id", "workspace-create"),
            ("channel_id", "channel-create"),
            ("actor_id", "operator"),
            ("memory_type", "fact"),
            ("importance", "0.75"),
            ("relation_target_id", "mem-create-related"),
            ("relation_type", "supports"),
            ("relation_weight", "0.42"),
        ])
        .send()
        .await
        .expect("submit memory create form");
    assert_eq!(create_response.status(), StatusCode::SEE_OTHER);
    let location = create_response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("ops memory create redirect location");
    assert!(location.contains("/ops/memory?"));
    assert!(location.contains("create_status=created"));
    assert!(location.contains("created_memory_id=mem-create-1"));

    let redirect_response = client
        .get(format!("http://{addr}{location}"))
        .send()
        .await
        .expect("load ops memory create redirect body");
    assert_eq!(redirect_response.status(), StatusCode::OK);
    let redirect_body = redirect_response
        .text()
        .await
        .expect("read ops memory create redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-memory-create-status\" data-create-status=\"created\" data-created-memory-id=\"mem-create-1\""
    ));

    let read_created_response = client
        .get(format!(
            "http://{addr}/gateway/memory/{session_key}/mem-create-1"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read created memory entry");
    assert_eq!(read_created_response.status(), StatusCode::OK);
    let read_created_payload: Value = read_created_response
        .json()
        .await
        .expect("parse created memory entry payload");
    assert_eq!(
        read_created_payload["entry"]["summary"].as_str(),
        Some("CreateToken summary")
    );
    assert_eq!(
        read_created_payload["entry"]["source_event_key"].as_str(),
        Some("evt-create-1")
    );
    assert_eq!(
        read_created_payload["entry"]["scope"]["workspace_id"].as_str(),
        Some("workspace-create")
    );
    assert_eq!(
        read_created_payload["entry"]["scope"]["channel_id"].as_str(),
        Some("channel-create")
    );
    assert_eq!(
        read_created_payload["entry"]["scope"]["actor_id"].as_str(),
        Some("operator")
    );
    assert_eq!(
        read_created_payload["entry"]["memory_type"].as_str(),
        Some("fact")
    );
    let importance = read_created_payload["entry"]["importance"]
        .as_f64()
        .expect("importance should be present for created entry");
    assert!(
        (importance - 0.75).abs() < f64::EPSILON,
        "importance should preserve create-form value"
    );
    assert_eq!(
        read_created_payload["entry"]["tags"],
        json!(["alpha", "beta"])
    );
    assert_eq!(
        read_created_payload["entry"]["facts"],
        json!(["fact-one", "fact-two"])
    );
    assert_eq!(
        read_created_payload["entry"]["relations"][0]["target_id"].as_str(),
        Some("mem-create-related")
    );

    let search_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=CreateToken&workspace_id=workspace-create&channel_id=channel-create&actor_id=operator&memory_type=fact"
        ))
        .send()
        .await
        .expect("query created memory through ops route");
    assert_eq!(search_response.status(), StatusCode::OK);
    let search_body = search_response
        .text()
        .await
        .expect("read memory search body");
    assert!(search_body.contains("data-memory-id=\"mem-create-1\" data-memory-type=\"fact\""));

    handle.abort();
}

#[tokio::test]
async fn regression_spec_2917_ops_memory_create_requires_entry_id_and_summary() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build client");

    let session_key = "ops-memory-create-required-fields";
    let missing_summary = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("entry_id", "mem-missing-summary"),
            ("summary", ""),
        ])
        .send()
        .await
        .expect("submit form with missing summary");
    assert_eq!(missing_summary.status(), StatusCode::SEE_OTHER);
    let missing_summary_location = missing_summary
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("missing-summary redirect location");
    assert!(missing_summary_location.contains("create_status=idle"));
    assert!(!missing_summary_location.contains("created_memory_id="));

    let read_missing_summary = client
        .get(format!(
            "http://{addr}/gateway/memory/{session_key}/mem-missing-summary"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read missing-summary memory entry");
    assert_eq!(read_missing_summary.status(), StatusCode::NOT_FOUND);

    let missing_entry_id = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("entry_id", ""),
            ("summary", "CreateToken should not persist without entry id"),
        ])
        .send()
        .await
        .expect("submit form with missing entry_id");
    assert_eq!(missing_entry_id.status(), StatusCode::SEE_OTHER);
    let missing_entry_location = missing_entry_id
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("missing-entry redirect location");
    assert!(missing_entry_location.contains("create_status=idle"));
    assert!(!missing_entry_location.contains("created_memory_id="));

    let redirect_body = client
        .get(format!("http://{addr}{missing_entry_location}"))
        .send()
        .await
        .expect("read missing-entry redirect body")
        .text()
        .await
        .expect("extract missing-entry redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-memory-create-status\" data-create-status=\"idle\" data-created-memory-id=\"\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_2921_c02_c03_ops_memory_edit_submission_updates_existing_entry_and_sets_status_markers(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state.clone())
        .await
        .expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build client");

    let session_key = "ops-memory-edit";
    let related_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-edit-related",
    );
    let related_create = client
        .put(format!("http://{addr}{related_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "EditToken relation target",
            "memory_type": "fact",
            "workspace_id": "workspace-edit",
            "channel_id": "channel-edit",
            "actor_id": "operator",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create related memory entry");
    assert_eq!(related_create.status(), StatusCode::CREATED);

    let target_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-edit-target",
    );
    let target_create = client
        .put(format!("http://{addr}{target_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "EditToken initial summary",
            "tags": ["alpha"],
            "facts": ["fact-initial"],
            "source_event_key": "evt-edit-initial",
            "workspace_id": "workspace-edit",
            "channel_id": "channel-edit",
            "actor_id": "operator",
            "memory_type": "fact",
            "importance": 0.88,
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create target memory entry");
    assert_eq!(target_create.status(), StatusCode::CREATED);

    let edit_response = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "edit"),
            ("entry_id", "mem-edit-target"),
            ("summary", "EditToken updated summary"),
            ("tags", "gamma,delta"),
            ("facts", "fact-updated-a|fact-updated-b"),
            ("source_event_key", "evt-edit-updated"),
            ("workspace_id", "workspace-edit"),
            ("channel_id", "channel-edit"),
            ("actor_id", "operator"),
            ("memory_type", "goal"),
            ("importance", "0.21"),
            ("relation_target_id", "mem-edit-related"),
            ("relation_type", "supports"),
            ("relation_weight", "0.32"),
        ])
        .send()
        .await
        .expect("submit memory edit form");
    assert_eq!(edit_response.status(), StatusCode::SEE_OTHER);
    let location = edit_response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("ops memory edit redirect location");
    assert!(location.contains("/ops/memory?"));
    assert!(location.contains("create_status=updated"));
    assert!(location.contains("created_memory_id=mem-edit-target"));

    let redirect_response = client
        .get(format!("http://{addr}{location}"))
        .send()
        .await
        .expect("load ops memory edit redirect body");
    assert_eq!(redirect_response.status(), StatusCode::OK);
    let redirect_body = redirect_response
        .text()
        .await
        .expect("read ops memory edit redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-memory-edit-status\" data-edit-status=\"updated\" data-edited-memory-id=\"mem-edit-target\""
    ));

    let read_updated_response = client
        .get(format!(
            "http://{addr}/gateway/memory/{session_key}/mem-edit-target"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read updated memory entry");
    assert_eq!(read_updated_response.status(), StatusCode::OK);
    let read_updated_payload: Value = read_updated_response
        .json()
        .await
        .expect("parse updated memory entry payload");
    assert_eq!(
        read_updated_payload["entry"]["summary"].as_str(),
        Some("EditToken updated summary")
    );
    assert_eq!(
        read_updated_payload["entry"]["source_event_key"].as_str(),
        Some("evt-edit-updated")
    );
    assert_eq!(
        read_updated_payload["entry"]["memory_type"].as_str(),
        Some("goal")
    );
    let importance = read_updated_payload["entry"]["importance"]
        .as_f64()
        .expect("importance should be present for updated entry");
    assert!(
        (importance - 0.21).abs() < 0.000_001,
        "importance should preserve edit-form value"
    );
    assert_eq!(
        read_updated_payload["entry"]["tags"],
        json!(["gamma", "delta"])
    );
    assert_eq!(
        read_updated_payload["entry"]["facts"],
        json!(["fact-updated-a", "fact-updated-b"])
    );
    assert_eq!(
        read_updated_payload["entry"]["relations"][0]["target_id"].as_str(),
        Some("mem-edit-related")
    );

    let search_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=EditToken&workspace_id=workspace-edit&channel_id=channel-edit&actor_id=operator&memory_type=goal"
        ))
        .send()
        .await
        .expect("query edited memory through ops route");
    assert_eq!(search_response.status(), StatusCode::OK);
    let search_body = search_response
        .text()
        .await
        .expect("read memory search body");
    assert!(search_body.contains("data-memory-id=\"mem-edit-target\" data-memory-type=\"goal\""));

    handle.abort();
}

#[tokio::test]
async fn regression_spec_2921_ops_memory_edit_requires_existing_entry() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build client");

    let session_key = "ops-memory-edit-required-existing";
    let edit_missing_entry = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "edit"),
            ("entry_id", "mem-edit-missing"),
            ("summary", "EditToken should not create from edit"),
        ])
        .send()
        .await
        .expect("submit form for missing entry");
    assert_eq!(edit_missing_entry.status(), StatusCode::SEE_OTHER);
    let location = edit_missing_entry
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("missing-entry edit redirect location");
    assert!(location.contains("create_status=idle"));
    assert!(!location.contains("created_memory_id="));

    let read_missing = client
        .get(format!(
            "http://{addr}/gateway/memory/{session_key}/mem-edit-missing"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read missing edit target");
    assert_eq!(read_missing.status(), StatusCode::NOT_FOUND);

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3060_c02_c03_ops_memory_delete_submission_requires_confirmation_and_deletes_confirmed_entry(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build client");

    let session_key = "ops-memory-delete";
    let target_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-delete-target",
    );
    let target_create = client
        .put(format!("http://{addr}{target_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "DeleteToken summary",
            "workspace_id": "workspace-delete",
            "channel_id": "channel-delete",
            "actor_id": "operator",
            "memory_type": "fact",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create target memory entry");
    assert_eq!(target_create.status(), StatusCode::CREATED);

    let missing_confirmation = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "delete"),
            ("entry_id", "mem-delete-target"),
            ("confirm_delete", "false"),
        ])
        .send()
        .await
        .expect("submit unconfirmed delete form");
    assert_eq!(missing_confirmation.status(), StatusCode::SEE_OTHER);
    let missing_confirmation_location = missing_confirmation
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("missing-confirmation redirect location");
    assert!(missing_confirmation_location.contains("/ops/memory?"));
    assert!(missing_confirmation_location.contains("delete_status=idle"));
    assert!(!missing_confirmation_location.contains("deleted_memory_id="));

    let still_present = client
        .get(format!(
            "http://{addr}/gateway/memory/{session_key}/mem-delete-target"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read target memory entry after unconfirmed delete");
    assert_eq!(still_present.status(), StatusCode::OK);

    let confirmed_delete = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "delete"),
            ("entry_id", "mem-delete-target"),
            ("confirm_delete", "true"),
        ])
        .send()
        .await
        .expect("submit confirmed delete form");
    assert_eq!(confirmed_delete.status(), StatusCode::SEE_OTHER);
    let confirmed_location = confirmed_delete
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("confirmed delete redirect location");
    assert!(confirmed_location.contains("/ops/memory?"));
    assert!(confirmed_location.contains("delete_status=deleted"));
    assert!(confirmed_location.contains("deleted_memory_id=mem-delete-target"));

    let redirect_body = client
        .get(format!("http://{addr}{confirmed_location}"))
        .send()
        .await
        .expect("load ops memory delete redirect body")
        .text()
        .await
        .expect("read ops memory delete redirect body");
    assert!(redirect_body.contains(
        "id=\"tau-ops-memory-delete-status\" data-delete-status=\"deleted\" data-deleted-memory-id=\"mem-delete-target\""
    ));

    let deleted_entry = client
        .get(format!(
            "http://{addr}/gateway/memory/{session_key}/mem-delete-target"
        ))
        .bearer_auth("secret")
        .send()
        .await
        .expect("read deleted memory entry");
    assert_eq!(deleted_entry.status(), StatusCode::NOT_FOUND);

    handle.abort();
}

#[tokio::test]
async fn regression_spec_3060_ops_memory_delete_requires_existing_entry_id() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build client");

    let session_key = "ops-memory-delete-required";
    let missing_entry = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "delete"),
            ("entry_id", ""),
            ("confirm_delete", "true"),
        ])
        .send()
        .await
        .expect("submit delete form without entry_id");
    assert_eq!(missing_entry.status(), StatusCode::SEE_OTHER);
    let missing_entry_location = missing_entry
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("missing-entry delete redirect location");
    assert!(missing_entry_location.contains("delete_status=idle"));
    assert!(!missing_entry_location.contains("deleted_memory_id="));

    let missing_target = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "delete"),
            ("entry_id", "mem-does-not-exist"),
            ("confirm_delete", "true"),
        ])
        .send()
        .await
        .expect("submit delete form for missing target");
    assert_eq!(missing_target.status(), StatusCode::SEE_OTHER);
    let missing_target_location = missing_target
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("missing-target delete redirect location");
    assert!(missing_target_location.contains("delete_status=idle"));
    assert!(!missing_target_location.contains("deleted_memory_id="));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3064_c02_c03_ops_memory_detail_panel_renders_embedding_and_relation_markers_for_selected_entry(
) {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-detail";
    let relation_target_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-detail-relation-target",
    );
    let relation_target_create = client
        .put(format!("http://{addr}{relation_target_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "DetailToken relation target",
            "workspace_id": "workspace-detail",
            "channel_id": "channel-detail",
            "actor_id": "operator",
            "memory_type": "fact",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create relation target entry");
    assert_eq!(relation_target_create.status(), StatusCode::CREATED);

    let detail_target_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-detail-target"),
            ("summary", "DetailToken primary entry"),
            ("workspace_id", "workspace-detail"),
            ("channel_id", "channel-detail"),
            ("actor_id", "operator"),
            ("memory_type", "goal"),
            ("relation_target_id", "mem-detail-relation-target"),
            ("relation_type", "supports"),
            ("relation_weight", "0.66"),
        ])
        .send()
        .await
        .expect("create detail target entry");
    assert_eq!(detail_target_create.status(), StatusCode::OK);

    let detail_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session={session_key}&query=DetailToken&workspace_id=workspace-detail&channel_id=channel-detail&actor_id=operator&memory_type=goal&detail_memory_id=mem-detail-target"
        ))
        .send()
        .await
        .expect("load ops memory detail route");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = detail_response
        .text()
        .await
        .expect("read ops memory detail body");

    assert!(detail_body.contains(
        "id=\"tau-ops-memory-detail-panel\" data-detail-visible=\"true\" data-memory-id=\"mem-detail-target\" data-memory-type=\"goal\""
    ));
    assert!(detail_body
        .contains("id=\"tau-ops-memory-detail-embedding\" data-embedding-source=\"hash-fnv1a\""));
    assert!(detail_body.contains("data-embedding-reason-code=\"memory_embedding_hash_only\""));
    assert!(detail_body.contains("id=\"tau-ops-memory-relations\" data-relation-count=\"1\""));
    assert!(detail_body.contains(
        "id=\"tau-ops-memory-relation-row-0\" data-target-id=\"mem-detail-relation-target\" data-relation-type=\"related_to\""
    ));

    handle.abort();
}

#[tokio::test]
async fn regression_spec_3064_ops_memory_detail_panel_hides_when_selected_entry_missing() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let detail_response = client
        .get(format!(
            "http://{addr}/ops/memory?theme=light&sidebar=collapsed&session=ops-memory-detail-missing&detail_memory_id=missing-entry"
        ))
        .send()
        .await
        .expect("load ops memory detail route with missing selection");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = detail_response
        .text()
        .await
        .expect("read ops memory detail missing-selection body");

    assert!(detail_body.contains(
        "id=\"tau-ops-memory-detail-panel\" data-detail-visible=\"false\" data-memory-id=\"\""
    ));
    assert!(detail_body.contains("id=\"tau-ops-memory-relations\" data-relation-count=\"0\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3068_c02_ops_memory_graph_route_renders_node_and_edge_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-graph";
    let target_endpoint = expand_memory_entry_template(
        GATEWAY_MEMORY_ENTRY_ENDPOINT,
        session_key,
        "mem-graph-target",
    );
    let target_create = client
        .put(format!("http://{addr}{target_endpoint}"))
        .bearer_auth("secret")
        .json(&json!({
            "summary": "Graph target",
            "workspace_id": "workspace-graph",
            "channel_id": "channel-graph",
            "actor_id": "operator",
            "memory_type": "fact",
            "policy_gate": MEMORY_WRITE_POLICY_GATE
        }))
        .send()
        .await
        .expect("create memory graph target entry");
    assert_eq!(target_create.status(), StatusCode::CREATED);

    let source_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-graph-source"),
            ("summary", "Graph source"),
            ("workspace_id", "workspace-graph"),
            ("channel_id", "channel-graph"),
            ("actor_id", "operator"),
            ("memory_type", "goal"),
            ("relation_target_id", "mem-graph-target"),
            ("relation_type", "supports"),
            ("relation_weight", "0.42"),
        ])
        .send()
        .await
        .expect("create memory graph source entry");
    assert_eq!(source_create.status(), StatusCode::OK);

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session={session_key}&workspace_id=workspace-graph&channel_id=channel-graph&actor_id=operator"
        ))
        .send()
        .await
        .expect("load ops memory graph route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.expect("read ops memory graph body");

    assert!(body.contains(
        "id=\"tau-ops-memory-graph-panel\" data-route=\"/ops/memory-graph\" aria-hidden=\"false\" data-panel-visible=\"true\""
    ));
    assert!(body.contains("id=\"tau-ops-memory-graph-nodes\" data-node-count=\"2\""));
    assert!(body.contains("id=\"tau-ops-memory-graph-edges\" data-edge-count=\"1\""));
    assert!(body.contains("data-memory-id=\"mem-graph-source\""));
    assert!(body.contains("data-memory-id=\"mem-graph-target\""));
    assert!(body.contains(
        "id=\"tau-ops-memory-graph-edge-0\" data-source-memory-id=\"mem-graph-source\" data-target-memory-id=\"mem-graph-target\""
    ));

    handle.abort();
}

#[tokio::test]
async fn regression_spec_3068_c03_non_memory_graph_routes_keep_hidden_graph_markers() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/chat?theme=light&sidebar=collapsed"
        ))
        .send()
        .await
        .expect("load non-memory-graph route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read non-memory-graph route body");

    assert!(body.contains(
        "id=\"tau-ops-memory-graph-panel\" data-route=\"/ops/memory-graph\" aria-hidden=\"true\" data-panel-visible=\"false\""
    ));
    assert!(body.contains("id=\"tau-ops-memory-graph-nodes\" data-node-count=\"0\""));
    assert!(body.contains("id=\"tau-ops-memory-graph-edges\" data-edge-count=\"0\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3070_c02_ops_memory_graph_node_size_markers_follow_importance() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-graph-size";
    let low_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-size-low"),
            ("summary", "Low importance"),
            ("workspace_id", "workspace-size"),
            ("channel_id", "channel-size"),
            ("actor_id", "operator"),
            ("memory_type", "fact"),
            ("importance", "0.10"),
        ])
        .send()
        .await
        .expect("create low-importance memory entry");
    assert_eq!(low_create.status(), StatusCode::OK);

    let high_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-size-high"),
            ("summary", "High importance"),
            ("workspace_id", "workspace-size"),
            ("channel_id", "channel-size"),
            ("actor_id", "operator"),
            ("memory_type", "goal"),
            ("importance", "0.90"),
        ])
        .send()
        .await
        .expect("create high-importance memory entry");
    assert_eq!(high_create.status(), StatusCode::OK);

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session={session_key}&workspace_id=workspace-size&channel_id=channel-size&actor_id=operator"
        ))
        .send()
        .await
        .expect("load ops memory graph size route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph size body");

    assert!(body.contains(
        "data-memory-id=\"mem-size-low\" data-memory-type=\"fact\" data-importance=\"0.1000\" data-node-size-bucket=\"small\" data-node-size-px=\"13.60\""
    ));
    assert!(body.contains(
        "data-memory-id=\"mem-size-high\" data-memory-type=\"goal\" data-importance=\"0.9000\" data-node-size-bucket=\"large\" data-node-size-px=\"26.40\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3078_c02_ops_memory_graph_node_color_markers_follow_memory_type() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-graph-color";
    let fact_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-color-fact"),
            ("summary", "Fact color"),
            ("workspace_id", "workspace-color"),
            ("channel_id", "channel-color"),
            ("actor_id", "operator"),
            ("memory_type", "fact"),
            ("importance", "0.50"),
        ])
        .send()
        .await
        .expect("create fact memory entry");
    assert_eq!(fact_create.status(), StatusCode::OK);

    let event_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-color-event"),
            ("summary", "Event color"),
            ("workspace_id", "workspace-color"),
            ("channel_id", "channel-color"),
            ("actor_id", "operator"),
            ("memory_type", "event"),
            ("importance", "0.50"),
        ])
        .send()
        .await
        .expect("create event memory entry");
    assert_eq!(event_create.status(), StatusCode::OK);

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session={session_key}&workspace_id=workspace-color&channel_id=channel-color&actor_id=operator"
        ))
        .send()
        .await
        .expect("load ops memory graph color route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph color body");

    assert!(body.contains(
        "data-memory-id=\"mem-color-fact\" data-memory-type=\"fact\" data-importance=\"0.5000\" data-node-size-bucket=\"medium\" data-node-size-px=\"20.00\" data-node-color-token=\"fact\" data-node-color-hex=\"#2563eb\""
    ));
    assert!(body.contains(
        "data-memory-id=\"mem-color-event\" data-memory-type=\"event\" data-importance=\"0.5000\" data-node-size-bucket=\"medium\" data-node-size-px=\"20.00\" data-node-color-token=\"event\" data-node-color-hex=\"#7c3aed\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3082_c02_ops_memory_graph_edge_style_markers_follow_relation_type() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-graph-edge-style";
    let target_rows = [
        ("mem-edge-target-0", "Target related"),
        ("mem-edge-target-1", "Target updates"),
        ("mem-edge-target-2", "Target contradicts"),
        ("mem-edge-target-3", "Target caused-by"),
    ];

    for (entry_id, summary) in target_rows {
        let create_response = client
            .post(format!("http://{addr}/ops/memory"))
            .form(&[
                ("theme", "light"),
                ("sidebar", "collapsed"),
                ("session", session_key),
                ("operation", "create"),
                ("entry_id", entry_id),
                ("summary", summary),
                ("workspace_id", "workspace-edge-style"),
                ("channel_id", "channel-edge-style"),
                ("actor_id", "operator"),
                ("memory_type", "fact"),
                ("importance", "0.50"),
            ])
            .send()
            .await
            .expect("create memory graph target row");
        assert_eq!(create_response.status(), StatusCode::OK);
    }

    let source_rows = [
        (
            "mem-edge-source-0",
            "Source related",
            "mem-edge-target-0",
            "supports",
        ),
        (
            "mem-edge-source-1",
            "Source updates",
            "mem-edge-target-1",
            "updates",
        ),
        (
            "mem-edge-source-2",
            "Source contradicts",
            "mem-edge-target-2",
            "contradicts",
        ),
        (
            "mem-edge-source-3",
            "Source caused-by",
            "mem-edge-target-3",
            "depends_on",
        ),
    ];

    for (entry_id, summary, relation_target_id, relation_type) in source_rows {
        let create_response = client
            .post(format!("http://{addr}/ops/memory"))
            .form(&[
                ("theme", "light"),
                ("sidebar", "collapsed"),
                ("session", session_key),
                ("operation", "create"),
                ("entry_id", entry_id),
                ("summary", summary),
                ("workspace_id", "workspace-edge-style"),
                ("channel_id", "channel-edge-style"),
                ("actor_id", "operator"),
                ("memory_type", "goal"),
                ("importance", "0.50"),
                ("relation_target_id", relation_target_id),
                ("relation_type", relation_type),
                ("relation_weight", "0.42"),
            ])
            .send()
            .await
            .expect("create memory graph source row");
        assert_eq!(create_response.status(), StatusCode::OK);
    }

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session={session_key}&workspace_id=workspace-edge-style&channel_id=channel-edge-style&actor_id=operator"
        ))
        .send()
        .await
        .expect("load ops memory graph edge style route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph edge style body");

    assert!(body.contains(
        "data-source-memory-id=\"mem-edge-source-0\" data-target-memory-id=\"mem-edge-target-0\" data-relation-type=\"related_to\" data-relation-weight=\"0.4200\" data-edge-style-token=\"solid\" data-edge-stroke-dasharray=\"none\""
    ));
    assert!(body.contains(
        "data-source-memory-id=\"mem-edge-source-1\" data-target-memory-id=\"mem-edge-target-1\" data-relation-type=\"updates\" data-relation-weight=\"0.4200\" data-edge-style-token=\"dashed\" data-edge-stroke-dasharray=\"6 4\""
    ));
    assert!(body.contains(
        "data-source-memory-id=\"mem-edge-source-2\" data-target-memory-id=\"mem-edge-target-2\" data-relation-type=\"contradicts\" data-relation-weight=\"0.4200\" data-edge-style-token=\"dotted\" data-edge-stroke-dasharray=\"2 4\""
    ));
    assert!(body.contains(
        "data-source-memory-id=\"mem-edge-source-3\" data-target-memory-id=\"mem-edge-target-3\" data-relation-type=\"caused_by\" data-relation-weight=\"0.4200\" data-edge-style-token=\"dashed\" data-edge-stroke-dasharray=\"6 4\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3086_c02_ops_memory_graph_selected_node_shows_detail_panel_contracts() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-graph-detail-panel";
    let selected_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-detail-graph"),
            ("summary", "Graph detail selected summary"),
            ("workspace_id", "workspace-detail-graph"),
            ("channel_id", "channel-detail-graph"),
            ("actor_id", "operator"),
            ("memory_type", "goal"),
            ("importance", "0.70"),
        ])
        .send()
        .await
        .expect("create selected graph memory entry");
    assert_eq!(selected_create.status(), StatusCode::OK);

    let other_create = client
        .post(format!("http://{addr}/ops/memory"))
        .form(&[
            ("theme", "light"),
            ("sidebar", "collapsed"),
            ("session", session_key),
            ("operation", "create"),
            ("entry_id", "mem-other-graph"),
            ("summary", "Graph detail unselected summary"),
            ("workspace_id", "workspace-detail-graph"),
            ("channel_id", "channel-detail-graph"),
            ("actor_id", "operator"),
            ("memory_type", "goal"),
            ("importance", "0.40"),
        ])
        .send()
        .await
        .expect("create unselected graph memory entry");
    assert_eq!(other_create.status(), StatusCode::OK);

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session={session_key}&workspace_id=workspace-detail-graph&channel_id=channel-detail-graph&actor_id=operator&memory_type=goal&detail_memory_id=mem-detail-graph"
        ))
        .send()
        .await
        .expect("load ops memory graph with selected detail");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph selected detail body");

    assert!(body.contains("id=\"tau-ops-memory-graph-node-0\" data-memory-id=\"mem-detail-graph\""));
    assert!(body.contains("id=\"tau-ops-memory-graph-node-1\" data-memory-id=\"mem-other-graph\""));
    assert!(body.contains("data-node-selected=\"true\""));
    assert!(body.contains("data-node-selected=\"false\""));
    assert!(body.contains("data-node-detail-href=\"/ops/memory-graph?theme=light"));
    assert!(body.contains("detail_memory_id=mem-detail-graph"));
    assert!(body.contains("detail_memory_id=mem-other-graph"));
    assert!(body.contains(
        "id=\"tau-ops-memory-graph-detail-panel\" data-detail-visible=\"true\" data-memory-id=\"mem-detail-graph\" data-memory-type=\"goal\" data-relation-count=\"0\""
    ));
    assert!(body.contains(
        "id=\"tau-ops-memory-graph-detail-summary\" data-memory-id=\"mem-detail-graph\">Graph detail selected summary"
    ));
    assert!(body
        .contains("id=\"tau-ops-memory-graph-detail-open-memory\" href=\"/ops/memory?theme=light"));
    assert!(body.contains("data-detail-memory-id=\"mem-detail-graph\""));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3090_c02_ops_memory_graph_focus_marks_connected_edges_and_neighbors() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let session_key = "ops-memory-graph-hover-focus";
    let entries = [
        ("mem-focus", "Focused memory", "goal", "0.70"),
        ("mem-neighbor", "Neighbor memory", "fact", "0.50"),
        ("mem-unrelated", "Unrelated memory", "event", "0.50"),
    ];
    for (entry_id, summary, memory_type, importance) in entries {
        let create_response = client
            .post(format!("http://{addr}/ops/memory"))
            .form(&[
                ("theme", "light"),
                ("sidebar", "collapsed"),
                ("session", session_key),
                ("operation", "create"),
                ("entry_id", entry_id),
                ("summary", summary),
                ("workspace_id", "workspace-hover"),
                ("channel_id", "channel-hover"),
                ("actor_id", "operator"),
                ("memory_type", memory_type),
                ("importance", importance),
            ])
            .send()
            .await
            .expect("create hover test memory entry");
        assert_eq!(create_response.status(), StatusCode::OK);
    }

    let relations = [
        ("mem-focus", "mem-neighbor", "supports", "0.42"),
        ("mem-neighbor", "mem-unrelated", "updates", "0.20"),
    ];
    for (entry_id, relation_target_id, relation_type, relation_weight) in relations {
        let relation_response = client
            .post(format!("http://{addr}/ops/memory"))
            .form(&[
                ("theme", "light"),
                ("sidebar", "collapsed"),
                ("session", session_key),
                ("operation", "edit"),
                ("entry_id", entry_id),
                ("summary", "Link relation"),
                ("workspace_id", "workspace-hover"),
                ("channel_id", "channel-hover"),
                ("actor_id", "operator"),
                ("memory_type", "goal"),
                ("importance", "0.70"),
                ("relation_target_id", relation_target_id),
                ("relation_type", relation_type),
                ("relation_weight", relation_weight),
            ])
            .send()
            .await
            .expect("add relation for hover test");
        assert_eq!(relation_response.status(), StatusCode::OK);
    }

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session={session_key}&workspace_id=workspace-hover&channel_id=channel-hover&actor_id=operator&detail_memory_id=mem-focus"
        ))
        .send()
        .await
        .expect("load ops memory graph hover focus route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph hover focus body");

    assert!(body.contains("data-memory-id=\"mem-focus\""));
    assert!(body.contains("data-node-hover-neighbor=\"true\""));
    assert!(body.contains(
        "data-source-memory-id=\"mem-focus\" data-target-memory-id=\"mem-neighbor\" data-relation-type=\"related_to\" data-relation-weight=\"0.4200\" data-edge-style-token=\"solid\" data-edge-stroke-dasharray=\"none\" data-edge-hover-highlighted=\"true\""
    ));
    assert!(body.contains(
        "data-source-memory-id=\"mem-neighbor\" data-target-memory-id=\"mem-unrelated\" data-relation-type=\"updates\" data-relation-weight=\"0.2000\" data-edge-style-token=\"dashed\" data-edge-stroke-dasharray=\"6 4\" data-edge-hover-highlighted=\"false\""
    ));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3094_c02_ops_memory_graph_zoom_query_clamps_and_updates_actions() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session=ops-zoom&workspace_id=workspace-zoom&channel_id=channel-zoom&actor_id=operator&memory_type=goal&graph_zoom=1.95"
        ))
        .send()
        .await
        .expect("load ops memory graph zoom route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph zoom body");

    assert!(body.contains(
        "id=\"tau-ops-memory-graph-zoom-controls\" data-zoom-level=\"1.95\" data-zoom-min=\"0.25\" data-zoom-max=\"2.00\" data-zoom-step=\"0.10\""
    ));
    assert!(body.contains("id=\"tau-ops-memory-graph-zoom-in\""));
    assert!(body.contains("data-zoom-action=\"in\""));
    assert!(body.contains("graph_zoom=2.00"));
    assert!(body.contains("id=\"tau-ops-memory-graph-zoom-out\""));
    assert!(body.contains("data-zoom-action=\"out\""));
    assert!(body.contains("graph_zoom=1.85"));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3099_c02_ops_memory_graph_pan_query_clamps_and_updates_actions() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session=ops-pan&workspace_id=workspace-pan&channel_id=channel-pan&actor_id=operator&memory_type=goal&graph_zoom=1.95&graph_pan_x=490&graph_pan_y=-495"
        ))
        .send()
        .await
        .expect("load ops memory graph pan route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph pan body");

    assert!(body.contains(
        "id=\"tau-ops-memory-graph-pan-controls\" data-pan-x=\"490.00\" data-pan-y=\"-495.00\" data-pan-step=\"25.00\""
    ));
    assert!(body.contains("id=\"tau-ops-memory-graph-pan-left\""));
    assert!(body.contains("data-pan-action=\"left\""));
    assert!(body.contains("graph_pan_x=465.00"));
    assert!(body.contains("id=\"tau-ops-memory-graph-pan-right\""));
    assert!(body.contains("data-pan-action=\"right\""));
    assert!(body.contains("graph_pan_x=500.00"));
    assert!(body.contains("id=\"tau-ops-memory-graph-pan-up\""));
    assert!(body.contains("data-pan-action=\"up\""));
    assert!(body.contains("graph_pan_y=-500.00"));
    assert!(body.contains("id=\"tau-ops-memory-graph-pan-down\""));
    assert!(body.contains("data-pan-action=\"down\""));
    assert!(body.contains("graph_pan_y=-470.00"));

    handle.abort();
}

#[tokio::test]
async fn integration_spec_3103_c02_ops_memory_graph_filter_query_updates_filter_contracts() {
    let temp = tempdir().expect("tempdir");
    let state = test_state(temp.path(), 10_000, "secret");
    let (addr, handle) = spawn_test_server(state).await.expect("spawn server");
    let client = Client::new();

    let response = client
        .get(format!(
            "http://{addr}/ops/memory-graph?theme=light&sidebar=collapsed&session=ops-filter&workspace_id=workspace-filter&channel_id=channel-filter&actor_id=operator&memory_type=goal&graph_zoom=1.25&graph_pan_x=25&graph_pan_y=-25&graph_filter_memory_type=goal&graph_filter_relation_type=related_to"
        ))
        .send()
        .await
        .expect("load ops memory graph filter route");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .text()
        .await
        .expect("read ops memory graph filter body");

    assert!(body.contains(
        "id=\"tau-ops-memory-graph-filter-controls\" data-filter-memory-type=\"goal\" data-filter-relation-type=\"related_to\""
    ));
    assert!(body.contains("id=\"tau-ops-memory-graph-filter-memory-type-all\""));
    assert!(body.contains("id=\"tau-ops-memory-graph-filter-memory-type-goal\""));
    assert!(body.contains("id=\"tau-ops-memory-graph-filter-relation-type-all\""));
    assert!(body.contains("id=\"tau-ops-memory-graph-filter-relation-type-related-to\""));
    assert!(body.contains("graph_filter_memory_type=all"));
    assert!(body.contains("graph_filter_memory_type=goal"));
    assert!(body.contains("graph_filter_relation_type=all"));
    assert!(body.contains("graph_filter_relation_type=related_to"));

    handle.abort();
}
