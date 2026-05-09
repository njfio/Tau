//! Harness self-improvement lineage overlay for gateway memory graph surfaces.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::{list_ops_harness_proposals, GatewayMemoryGraphEdge, GatewayMemoryGraphNode};

pub(super) fn append_ops_harness_memory_graph_lineage(
    nodes: &mut Vec<GatewayMemoryGraphNode>,
    edges: &mut Vec<GatewayMemoryGraphEdge>,
    state_dir: &Path,
    relation_type_filter: Option<&BTreeSet<String>>,
    min_edge_weight: f64,
) {
    let (lineage_nodes, lineage_edges) =
        collect_ops_harness_memory_graph_lineage(state_dir, relation_type_filter, min_edge_weight);
    append_gateway_graph_rows(nodes, edges, lineage_nodes, lineage_edges);
}

pub(super) fn collect_ops_harness_memory_graph_lineage(
    state_dir: &Path,
    relation_type_filter: Option<&BTreeSet<String>>,
    min_edge_weight: f64,
) -> (Vec<GatewayMemoryGraphNode>, Vec<GatewayMemoryGraphEdge>) {
    let self_improvement_root = state_dir.join("ops-harness").join("self-improvement");
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut edge_ids = BTreeSet::new();

    for proposal_definition in list_ops_harness_proposals() {
        let mission_path = self_improvement_root
            .join(proposal_definition.proposal_id)
            .join("mission.json");
        let Ok(mission_json) = std::fs::read_to_string(&mission_path) else {
            continue;
        };
        let Ok(mission_state) = serde_json::from_str::<Value>(&mission_json) else {
            continue;
        };
        let mission_id = mission_state
            .get("mission_id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(proposal_definition.mission_id);
        let mission_label = mission_state
            .get("goal")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(proposal_definition.goal);
        add_gateway_graph_node(
            &mut nodes,
            &mut node_ids,
            mission_id,
            mission_label,
            "goal",
            0.95,
        );

        if let Some(records) = mission_state
            .get("learning_records")
            .and_then(Value::as_array)
        {
            for record in records {
                let Some(record_id) = record.get("record_id").and_then(Value::as_str) else {
                    continue;
                };
                let record_label = record
                    .get("summary")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(record_id);
                add_gateway_graph_node(
                    &mut nodes,
                    &mut node_ids,
                    record_id,
                    record_label,
                    "observation",
                    0.85,
                );
                add_gateway_graph_edge(
                    &mut edges,
                    &mut edge_ids,
                    relation_type_filter,
                    min_edge_weight,
                    mission_id,
                    record_id,
                    "contains",
                    1.0,
                );
            }
        }

        if let Some(proposals) = mission_state
            .get("improvement_proposals")
            .and_then(Value::as_array)
        {
            for proposal in proposals {
                let Some(proposal_id) = proposal.get("proposal_id").and_then(Value::as_str) else {
                    continue;
                };
                let source_learning_record_id = proposal
                    .get("source_learning_record_id")
                    .and_then(Value::as_str)
                    .unwrap_or(proposal_definition.source_learning_record_id);
                let target_path = proposal
                    .get("target_path")
                    .and_then(Value::as_str)
                    .unwrap_or(proposal_definition.target_path);
                let target_node_id = format!("target:{target_path}");
                let proposal_label = proposal
                    .get("patch_summary")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(proposal_definition.patch_summary);
                add_gateway_graph_node(
                    &mut nodes,
                    &mut node_ids,
                    proposal_id,
                    proposal_label,
                    "event",
                    0.90,
                );
                add_gateway_graph_node(
                    &mut nodes,
                    &mut node_ids,
                    target_node_id.as_str(),
                    target_path,
                    "fact",
                    0.65,
                );
                add_gateway_graph_edge(
                    &mut edges,
                    &mut edge_ids,
                    relation_type_filter,
                    min_edge_weight,
                    source_learning_record_id,
                    proposal_id,
                    "supports",
                    1.0,
                );
                add_gateway_graph_edge(
                    &mut edges,
                    &mut edge_ids,
                    relation_type_filter,
                    min_edge_weight,
                    proposal_id,
                    target_node_id.as_str(),
                    "updates",
                    1.0,
                );

                if proposal
                    .get("dry_run")
                    .is_some_and(|dry_run| !dry_run.is_null())
                {
                    let dry_run_node_id = format!("dry-run:{proposal_id}");
                    add_gateway_graph_node(
                        &mut nodes,
                        &mut node_ids,
                        dry_run_node_id.as_str(),
                        "Dry-run evidence recorded",
                        "event",
                        0.70,
                    );
                    add_gateway_graph_edge(
                        &mut edges,
                        &mut edge_ids,
                        relation_type_filter,
                        min_edge_weight,
                        proposal_id,
                        dry_run_node_id.as_str(),
                        "result_of",
                        1.0,
                    );
                }

                if proposal
                    .get("applied_unix_ms")
                    .is_some_and(|applied| !applied.is_null())
                {
                    let apply_node_id = format!("apply:{proposal_id}");
                    add_gateway_graph_node(
                        &mut nodes,
                        &mut node_ids,
                        apply_node_id.as_str(),
                        "Apply evidence recorded",
                        "event",
                        0.75,
                    );
                    add_gateway_graph_edge(
                        &mut edges,
                        &mut edge_ids,
                        relation_type_filter,
                        min_edge_weight,
                        dry_run_or_proposal_node(proposal, proposal_id).as_str(),
                        apply_node_id.as_str(),
                        "supports",
                        1.0,
                    );
                }
            }
        }
    }

    (nodes, edges)
}

fn append_gateway_graph_rows(
    nodes: &mut Vec<GatewayMemoryGraphNode>,
    edges: &mut Vec<GatewayMemoryGraphEdge>,
    node_rows: Vec<GatewayMemoryGraphNode>,
    edge_rows: Vec<GatewayMemoryGraphEdge>,
) {
    let mut node_ids = nodes
        .iter()
        .map(|row| row.id.clone())
        .collect::<BTreeSet<_>>();
    for row in node_rows {
        if node_ids.insert(row.id.clone()) {
            nodes.push(row);
        }
    }

    let mut edge_ids = edges
        .iter()
        .map(|row| row.id.clone())
        .collect::<BTreeSet<_>>();
    for row in edge_rows {
        if edge_ids.insert(row.id.clone()) {
            edges.push(row);
        }
    }

    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    edges.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then(left.target.cmp(&right.target))
            .then(left.relation_type.cmp(&right.relation_type))
            .then(left.id.cmp(&right.id))
    });
}

fn dry_run_or_proposal_node(proposal: &Value, proposal_id: &str) -> String {
    if proposal
        .get("dry_run")
        .is_some_and(|dry_run| !dry_run.is_null())
    {
        format!("dry-run:{proposal_id}")
    } else {
        proposal_id.to_string()
    }
}

fn add_gateway_graph_node(
    nodes: &mut Vec<GatewayMemoryGraphNode>,
    node_ids: &mut BTreeSet<String>,
    id: &str,
    label: &str,
    category: &str,
    weight: f64,
) {
    if !node_ids.insert(id.to_string()) {
        return;
    }
    nodes.push(GatewayMemoryGraphNode {
        id: id.to_string(),
        label: label.to_string(),
        category: category.to_string(),
        weight,
        size: 12.0 + (weight.clamp(0.0, 1.0) * 16.0),
    });
}

#[allow(clippy::too_many_arguments)]
fn add_gateway_graph_edge(
    edges: &mut Vec<GatewayMemoryGraphEdge>,
    edge_ids: &mut BTreeSet<String>,
    relation_type_filter: Option<&BTreeSet<String>>,
    min_edge_weight: f64,
    source: &str,
    target: &str,
    relation_type: &str,
    weight: f64,
) {
    if relation_type_filter.is_some_and(|filter| !filter.contains(relation_type))
        || weight < min_edge_weight
    {
        return;
    }
    let id = format!("edge:harness:{source}:{target}:{relation_type}");
    if !edge_ids.insert(id.clone()) {
        return;
    }
    edges.push(GatewayMemoryGraphEdge {
        id,
        source: source.to_string(),
        target: target.to_string(),
        relation_type: relation_type.to_string(),
        weight,
    });
}
