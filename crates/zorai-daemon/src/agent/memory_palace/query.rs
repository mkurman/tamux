use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::Result;

use crate::history::HistoryStore;

use super::types::{MemoryPalaceEdge, MemoryPalaceGraphContext, MemoryPalaceNode};

pub(crate) async fn query_memory_graph(
    history: &HistoryStore,
    center_node_id: &str,
    depth: usize,
    per_hop_limit: usize,
) -> Result<MemoryPalaceGraphContext> {
    let center = history
        .get_memory_node(center_node_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("memory palace center node `{center_node_id}` not found"))?;

    let mut nodes = BTreeMap::new();
    let mut edges = Vec::new();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([(center_node_id.to_string(), 0usize)]);

    nodes.insert(
        center.id.clone(),
        MemoryPalaceNode {
            id: center.id.clone(),
            label: center.label,
            node_type: center.node_type,
            summary_text: center.summary_text,
        },
    );
    visited.insert(center_node_id.to_string());

    while let Some((node_id, current_depth)) = queue.pop_front() {
        if current_depth >= depth {
            continue;
        }
        for neighbor in history
            .list_memory_graph_neighbors(&node_id, per_hop_limit.max(1))
            .await?
        {
            nodes
                .entry(neighbor.node.id.clone())
                .or_insert_with(|| MemoryPalaceNode {
                    id: neighbor.node.id.clone(),
                    label: neighbor.node.label.clone(),
                    node_type: neighbor.node.node_type.clone(),
                    summary_text: neighbor.node.summary_text.clone(),
                });
            edges.push(MemoryPalaceEdge {
                source_node_id: neighbor.via_edge.source_node_id.clone(),
                target_node_id: neighbor.via_edge.target_node_id.clone(),
                relation: neighbor.via_edge.relation_type.clone(),
                weight: neighbor.via_edge.weight,
            });
            if visited.insert(neighbor.node.id.clone()) {
                queue.push_back((neighbor.node.id, current_depth + 1));
            }
        }
    }

    let visited_node_ids: Vec<String> = visited.iter().cloned().collect();
    let cluster_summaries = history
        .list_memory_cluster_summaries_for_nodes(&visited_node_ids, 4)
        .await?;

    let summary = edges
        .iter()
        .take(6)
        .map(|edge| {
            let source = nodes
                .get(&edge.source_node_id)
                .map(|node| node.label.as_str())
                .unwrap_or(edge.source_node_id.as_str());
            let target = nodes
                .get(&edge.target_node_id)
                .map(|node| node.label.as_str())
                .unwrap_or(edge.target_node_id.as_str());
            format!("{source} -[{}]-> {target}", edge.relation)
        })
        .collect::<Vec<_>>()
        .join("; ");
    let summary = if cluster_summaries.is_empty() {
        summary
    } else if summary.is_empty() {
        cluster_summaries.join("; ")
    } else {
        format!("{summary}; {}", cluster_summaries.join("; "))
    };

    Ok(MemoryPalaceGraphContext {
        center_node_id: center_node_id.to_string(),
        subgraph_nodes: nodes.into_values().collect(),
        subgraph_edges: edges,
        cluster_summaries,
        summary,
    })
}
