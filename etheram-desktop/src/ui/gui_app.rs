// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::launcher::LaunchedNode;
use crate::launcher::Launcher;
use crate::launcher::NodeLogLine;
use eframe::egui;
use eframe::run_native;
use eframe::App;
use eframe::Frame;
use eframe::NativeOptions;
use etheram_node_process::cluster_config::ClusterConfig;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;

pub fn run_gui(initial_cluster_path: Option<String>) -> Result<(), String> {
    let app = DesktopGuiApp::new(initial_cluster_path);
    let native_options = NativeOptions::default();
    run_native(
        "EtheRAM Desktop",
        native_options,
        Box::new(move |_cc| Ok(Box::new(app))),
    )
    .map_err(|error| format!("failed to start desktop GUI: {error}"))
}

struct DesktopGuiApp {
    cluster_path: String,
    from_peer: String,
    to_peer: String,
    isolate_peer: String,
    launched_nodes: Option<Vec<LaunchedNode>>,
    log_lines: Vec<UiLogLine>,
    log_filter_node: String,
    log_filter_text: String,
    log_receiver: Option<Receiver<NodeLogLine>>,
    blocked_links: BTreeSet<(u64, u64)>,
    latest_node_status: BTreeMap<u64, NodeStatusSnapshot>,
    status_line: String,
}

struct UiLogLine {
    node_id: u64,
    line: String,
}

#[derive(Clone)]
struct NodeStatusSnapshot {
    height: u64,
    last_hash: String,
}

impl DesktopGuiApp {
    fn new(initial_cluster_path: Option<String>) -> Self {
        Self {
            cluster_path: initial_cluster_path.unwrap_or_default(),
            from_peer: String::new(),
            to_peer: String::new(),
            isolate_peer: String::new(),
            launched_nodes: None,
            log_lines: Vec::new(),
            log_filter_node: String::new(),
            log_filter_text: String::new(),
            log_receiver: None,
            blocked_links: BTreeSet::new(),
            latest_node_status: BTreeMap::new(),
            status_line: String::new(),
        }
    }

    fn node_color(node_id: u64) -> egui::Color32 {
        match node_id % 5 {
            0 => egui::Color32::from_rgb(180, 180, 180),
            1 => egui::Color32::from_rgb(170, 220, 170),
            2 => egui::Color32::from_rgb(170, 200, 240),
            3 => egui::Color32::from_rgb(240, 200, 150),
            _ => egui::Color32::from_rgb(210, 180, 230),
        }
    }

    fn line_color(node_id: u64, line: &str) -> egui::Color32 {
        if line.contains("error") {
            return egui::Color32::from_rgb(220, 80, 80);
        }
        if line.contains("desktop_control") {
            return egui::Color32::from_rgb(230, 210, 120);
        }
        if line.contains("partition_update") {
            return egui::Color32::from_rgb(230, 180, 80);
        }
        if line.contains("node_status") {
            let base = Self::node_color(node_id);
            let red = ((base.r() as u16 + 120) / 2) as u8;
            let green = ((base.g() as u16 + 180) / 2) as u8;
            let blue = ((base.b() as u16 + 240) / 2) as u8;
            return egui::Color32::from_rgb(red, green, blue);
        }
        if line.contains("bootstrap") {
            return egui::Color32::from_rgb(120, 200, 140);
        }
        if line.contains("loop mode") {
            return egui::Color32::from_rgb(120, 200, 200);
        }
        Self::node_color(node_id)
    }

    fn push_ui_log(&mut self, line: String) {
        self.log_lines.push(UiLogLine { node_id: 0, line });
        if self.log_lines.len() > 500 {
            let overflow = self.log_lines.len() - 500;
            self.log_lines.drain(0..overflow);
        }
    }

    fn log_visible(&self, entry: &UiLogLine) -> bool {
        let node_filter = self.log_filter_node.trim();
        if !node_filter.is_empty() {
            let Ok(node_id) = node_filter.parse::<u64>() else {
                return false;
            };
            if entry.node_id != node_id {
                return false;
            }
        }

        let text_filter = self.log_filter_text.trim().to_lowercase();
        if text_filter.is_empty() {
            return true;
        }

        entry.line.to_lowercase().contains(&text_filter)
    }

    fn drain_logs(&mut self) {
        let Some(receiver) = self.log_receiver.as_ref() else {
            return;
        };

        let mut drained_entries = Vec::new();

        loop {
            match receiver.try_recv() {
                Ok(entry) => {
                    drained_entries.push(entry);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.log_receiver = None;
                    break;
                }
            }
        }

        for entry in drained_entries {
            self.apply_partition_update_from_line(&entry.line);
            self.apply_node_status_from_line(&entry.line);
            self.log_lines.push(UiLogLine {
                node_id: entry.node_id,
                line: entry.line,
            });
            if self.log_lines.len() > 500 {
                let overflow = self.log_lines.len() - 500;
                self.log_lines.drain(0..overflow);
            }
        }
    }

    fn apply_node_status_from_line(&mut self, line: &str) {
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

    fn convergence_status(&self, running_nodes: usize) -> (String, egui::Color32) {
        if running_nodes == 0 {
            return (
                "Cluster Convergence: n/a (cluster stopped)".to_string(),
                egui::Color32::from_rgb(180, 180, 180),
            );
        }

        if self.latest_node_status.len() < running_nodes {
            return (
                format!(
                    "Cluster Convergence: waiting for node_status ({}/{})",
                    self.latest_node_status.len(),
                    running_nodes
                ),
                egui::Color32::from_rgb(220, 180, 90),
            );
        }

        let mut snapshots = self.latest_node_status.values();
        let Some(first) = snapshots.next() else {
            return (
                "Cluster Convergence: waiting for node_status".to_string(),
                egui::Color32::from_rgb(220, 180, 90),
            );
        };

        let converged =
            snapshots.all(|item| item.height == first.height && item.last_hash == first.last_hash);
        if converged {
            return (
                format!(
                    "Cluster Convergence: converged (height={} hash={})",
                    first.height, first.last_hash
                ),
                egui::Color32::from_rgb(120, 200, 140),
            );
        }

        let summary = self
            .latest_node_status
            .iter()
            .map(|(node_id, item)| format!("{}:{}", node_id, item.height))
            .collect::<Vec<String>>()
            .join(", ");
        (
            format!("Cluster Convergence: diverged ({summary})"),
            egui::Color32::from_rgb(220, 80, 80),
        )
    }

    fn apply_partition_update_from_line(&mut self, line: &str) {
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

    fn parse_peer_input(value: &str, field: &str) -> Result<u64, String> {
        value
            .trim()
            .parse::<u64>()
            .map_err(|error| format!("invalid {field} peer id '{value}': {error}"))
    }

    fn with_nodes_mut<F>(&mut self, operation: F) -> bool
    where
        F: FnOnce(&mut [LaunchedNode]) -> Result<(), String>,
    {
        match self.launched_nodes.as_mut() {
            Some(nodes) => match operation(nodes) {
                Ok(()) => {
                    self.status_line = "command sent".to_string();
                    true
                }
                Err(error) => {
                    self.status_line = error;
                    false
                }
            },
            None => {
                self.status_line = "cluster is not running".to_string();
                false
            }
        }
    }

    fn start_cluster(&mut self) {
        if self.launched_nodes.is_some() {
            self.status_line = "cluster already running".to_string();
            return;
        }

        let config = match ClusterConfig::load_from_path(Path::new(&self.cluster_path)) {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };

        match Launcher::spawn_node_processes(&config, &self.cluster_path) {
            Ok(mut nodes) => {
                self.log_lines.clear();
                self.blocked_links.clear();
                self.latest_node_status.clear();
                self.log_receiver = Some(Launcher::start_log_pump(&mut nodes));
                self.launched_nodes = Some(nodes);
                self.status_line = "cluster started".to_string();
            }
            Err(error) => {
                self.status_line = error;
            }
        }
    }

    fn stop_cluster(&mut self) {
        if let Some(nodes) = self.launched_nodes.take() {
            self.log_receiver = None;
            self.blocked_links.clear();
            self.latest_node_status.clear();
            match Launcher::stop_all(nodes) {
                Ok(()) => {
                    self.status_line = "cluster stopped".to_string();
                }
                Err(error) => {
                    self.status_line = error;
                }
            }
        } else {
            self.status_line = "cluster is not running".to_string();
        }
    }

    fn send_partition(&mut self) {
        let from = match Self::parse_peer_input(&self.from_peer, "from") {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };
        let to = match Self::parse_peer_input(&self.to_peer, "to") {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };
        if self.with_nodes_mut(|nodes| Launcher::broadcast_partition_command(nodes, from, to)) {
            self.blocked_links.insert((from, to));
            self.push_ui_log(format!(
                "desktop_control partition from_peer={} to_peer={}",
                from, to
            ));
        }
    }

    fn send_heal(&mut self) {
        let from = match Self::parse_peer_input(&self.from_peer, "from") {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };
        let to = match Self::parse_peer_input(&self.to_peer, "to") {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };
        if self.with_nodes_mut(|nodes| Launcher::broadcast_heal_command(nodes, from, to)) {
            self.blocked_links.remove(&(from, to));
            self.push_ui_log(format!(
                "desktop_control heal from_peer={} to_peer={}",
                from, to
            ));
        }
    }

    fn send_clear(&mut self) {
        if self.with_nodes_mut(Launcher::broadcast_clear_command) {
            self.blocked_links.clear();
            self.push_ui_log("desktop_control clear".to_string());
        }
    }

    fn send_isolate_node(&mut self) {
        let target = match Self::parse_peer_input(&self.isolate_peer, "node") {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };

        let mut link_count = 0usize;
        if self.with_nodes_mut(|nodes| {
            let count = Launcher::broadcast_isolate_node_command(nodes, target)?;
            link_count = count;
            Ok(())
        }) {
            for node_id in self.latest_node_status.keys().copied() {
                if node_id == target {
                    continue;
                }
                self.blocked_links.insert((target, node_id));
                self.blocked_links.insert((node_id, target));
            }
            self.push_ui_log(format!(
                "desktop_control isolate node_peer={} links_blocked={}",
                target, link_count
            ));
        }
    }

    fn send_heal_isolated_node(&mut self) {
        let target = match Self::parse_peer_input(&self.isolate_peer, "node") {
            Ok(value) => value,
            Err(error) => {
                self.status_line = error;
                return;
            }
        };

        let mut link_count = 0usize;
        if self.with_nodes_mut(|nodes| {
            let count = Launcher::broadcast_heal_isolated_node_command(nodes, target)?;
            link_count = count;
            Ok(())
        }) {
            for node_id in self.latest_node_status.keys().copied() {
                if node_id == target {
                    continue;
                }
                self.blocked_links.remove(&(target, node_id));
                self.blocked_links.remove(&(node_id, target));
            }
            self.push_ui_log(format!(
                "desktop_control heal_isolated node_peer={} links_healed={}",
                target, link_count
            ));
        }
    }

    fn send_shutdown(&mut self) {
        if self.with_nodes_mut(Launcher::broadcast_shutdown_command) {
            self.push_ui_log("desktop_control shutdown".to_string());
        }
        self.stop_cluster();
    }

    fn copy_all_logs(&mut self, ctx: &egui::Context) {
        let text = self
            .log_lines
            .iter()
            .map(|entry| format!("node={} {}", entry.node_id, entry.line))
            .collect::<Vec<String>>()
            .join("\n");
        ctx.copy_text(text);
        self.status_line = "all logs copied".to_string();
    }
}

impl Drop for DesktopGuiApp {
    fn drop(&mut self) {
        if let Some(nodes) = self.launched_nodes.take() {
            let _ = Launcher::stop_all(nodes);
        }
    }
}

impl App for DesktopGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.drain_logs();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("EtheRAM Desktop Control");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Cluster Config:");
                ui.text_edit_singleline(&mut self.cluster_path);
            });

            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    self.start_cluster();
                }
                if ui.button("Stop").clicked() {
                    self.stop_cluster();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("From:");
                ui.text_edit_singleline(&mut self.from_peer);
                ui.label("To:");
                ui.text_edit_singleline(&mut self.to_peer);
            });

            ui.horizontal(|ui| {
                ui.label("Node:");
                ui.text_edit_singleline(&mut self.isolate_peer);
                if ui.button("Isolate Node (Both Directions)").clicked() {
                    self.send_isolate_node();
                }
                if ui.button("Heal Isolated Node").clicked() {
                    self.send_heal_isolated_node();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Partition").clicked() {
                    self.send_partition();
                }
                if ui.button("Heal").clicked() {
                    self.send_heal();
                }
                if ui.button("Clear").clicked() {
                    self.send_clear();
                }
                if ui.button("Shutdown").clicked() {
                    self.send_shutdown();
                }
                if ui.button("Copy All Logs").clicked() {
                    self.copy_all_logs(ctx);
                }
            });

            ui.separator();
            ui.label(format!("Status: {}", self.status_line));
            let running_nodes = self.launched_nodes.as_ref().map_or(0, |nodes| nodes.len());
            ui.label(format!("Running Nodes: {}", running_nodes));
            let (convergence_text, convergence_color) = self.convergence_status(running_nodes);
            ui.label(egui::RichText::new(convergence_text).color(convergence_color));
            let partition_active = !self.blocked_links.is_empty();
            let partition_text = if partition_active {
                format!(
                    "Partition Active: yes ({} link{})",
                    self.blocked_links.len(),
                    if self.blocked_links.len() == 1 { "" } else { "s" }
                )
            } else {
                "Partition Active: no".to_string()
            };
            let partition_color = if partition_active {
                egui::Color32::from_rgb(220, 80, 80)
            } else {
                egui::Color32::from_rgb(120, 200, 140)
            };
            ui.label(egui::RichText::new(partition_text).color(partition_color));
            if partition_active {
                let links = self
                    .blocked_links
                    .iter()
                    .map(|(from, to)| format!("{}->{}", from, to))
                    .collect::<Vec<String>>()
                    .join(", ");
                ui.label(format!("Blocked Links: {}", links));
            }

            ui.separator();
            ui.label("Node Logs:");
            ui.horizontal(|ui| {
                ui.label("Node Filter:");
                ui.text_edit_singleline(&mut self.log_filter_node);
                ui.label("Text Filter:");
                ui.text_edit_singleline(&mut self.log_filter_text);
            });
            ui.label(
                "Legend: red=error yellow=desktop amber=partition status=tinted-by-node green=bootstrap cyan=loop mode",
            );
            egui::ScrollArea::vertical()
                .max_height(260.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &self.log_lines {
                        if !self.log_visible(entry) {
                            continue;
                        }
                        let formatted = format!("node={} {}", entry.node_id, entry.line);
                        ui.label(
                            egui::RichText::new(formatted)
                                .color(Self::line_color(entry.node_id, &entry.line)),
                        );
                    }
                });
        });
    }
}
