// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct NodeStatusSnapshot {
    pub height: u64,
    pub last_hash: String,
}

pub enum ConvergenceStatus {
    Stopped,
    Waiting { have: usize, want: usize },
    Converged { height: u64, hash: String },
    Diverged { summary: String },
}

impl ConvergenceStatus {
    pub fn label(&self) -> String {
        match self {
            Self::Stopped => "Cluster Convergence: n/a (cluster stopped)".to_string(),
            Self::Waiting { have, want } => {
                format!("Cluster Convergence: waiting for node_status ({have}/{want})")
            }
            Self::Converged { height, hash } => {
                format!("Cluster Convergence: converged (height={height} hash={hash})")
            }
            Self::Diverged { summary } => {
                format!("Cluster Convergence: diverged ({summary})")
            }
        }
    }
}

pub struct ClusterState {
    pub blocked_links: BTreeSet<(u64, u64)>,
    pub latest_node_status: BTreeMap<u64, NodeStatusSnapshot>,
}

impl ClusterState {
    pub fn new() -> Self {
        Self {
            blocked_links: BTreeSet::new(),
            latest_node_status: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.blocked_links.clear();
        self.latest_node_status.clear();
    }

    pub fn apply_node_status_from_line(&mut self, line: &str) {
        if !line.contains("node_status") {
            return;
        }

        let mut peer_id: Option<u64> = None;
        let mut height: Option<u64> = None;
        let mut last_hash: Option<String> = None;

        for token in line.split_whitespace() {
            if let Some(value) = token.strip_prefix("peer_id=") {
                peer_id = value.parse::<u64>().ok();
            }
            if let Some(value) = token.strip_prefix("height=") {
                height = value.parse::<u64>().ok();
            }
            if let Some(value) = token.strip_prefix("last_hash=") {
                last_hash = Some(value.to_string());
            }
        }

        let (Some(peer_id), Some(height), Some(last_hash)) = (peer_id, height, last_hash) else {
            return;
        };

        self.latest_node_status
            .insert(peer_id, NodeStatusSnapshot { height, last_hash });
    }

    pub fn apply_partition_update_from_line(&mut self, line: &str) {
        if !line.contains("partition_update") {
            return;
        }

        if line.contains("cleared") {
            self.blocked_links.clear();
            return;
        }

        let mut from_peer: Option<u64> = None;
        let mut to_peer: Option<u64> = None;

        for token in line.split_whitespace() {
            if let Some(value) = token.strip_prefix("from_peer=") {
                from_peer = value.parse::<u64>().ok();
            }
            if let Some(value) = token.strip_prefix("to_peer=") {
                to_peer = value.parse::<u64>().ok();
            }
        }

        let (Some(from), Some(to)) = (from_peer, to_peer) else {
            return;
        };

        if line.contains("blocked") {
            self.blocked_links.insert((from, to));
            return;
        }

        if line.contains("healed") {
            self.blocked_links.remove(&(from, to));
        }
    }

    pub fn convergence_status(&self, running_nodes: usize) -> ConvergenceStatus {
        if running_nodes == 0 {
            return ConvergenceStatus::Stopped;
        }

        if self.latest_node_status.len() < running_nodes {
            return ConvergenceStatus::Waiting {
                have: self.latest_node_status.len(),
                want: running_nodes,
            };
        }

        let mut snapshots = self.latest_node_status.values();
        let Some(first) = snapshots.next() else {
            return ConvergenceStatus::Waiting {
                have: 0,
                want: running_nodes,
            };
        };

        let converged =
            snapshots.all(|item| item.height == first.height && item.last_hash == first.last_hash);
        if converged {
            return ConvergenceStatus::Converged {
                height: first.height,
                hash: first.last_hash.clone(),
            };
        }

        let summary = self
            .latest_node_status
            .iter()
            .map(|(node_id, item)| format!("{}:{}", node_id, item.height))
            .collect::<Vec<String>>()
            .join(", ");
        ConvergenceStatus::Diverged { summary }
    }
}

impl Default for ClusterState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn log_visible(filter_node: &str, filter_text: &str, node_id: u64, line: &str) -> bool {
    let node_filter = filter_node.trim();
    if !node_filter.is_empty() {
        let Ok(filter_id) = node_filter.parse::<u64>() else {
            return false;
        };
        if node_id != filter_id {
            return false;
        }
    }

    let text_filter = filter_text.trim().to_lowercase();
    if text_filter.is_empty() {
        return true;
    }

    line.to_lowercase().contains(&text_filter)
}
