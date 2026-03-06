// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_config::ClusterConfig;
use crate::launcher::LaunchedNode;
use crate::launcher::Launcher;
use crate::launcher::NodeLogLine;
use eframe::egui;
use eframe::run_native;
use eframe::App;
use eframe::Frame;
use eframe::NativeOptions;
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
    launched_nodes: Option<Vec<LaunchedNode>>,
    log_lines: Vec<UiLogLine>,
    log_filter_node: String,
    log_filter_text: String,
    log_receiver: Option<Receiver<NodeLogLine>>,
    status_line: String,
}

struct UiLogLine {
    node_id: u64,
    line: String,
}

impl DesktopGuiApp {
    fn new(initial_cluster_path: Option<String>) -> Self {
        Self {
            cluster_path: initial_cluster_path.unwrap_or_default(),
            from_peer: String::new(),
            to_peer: String::new(),
            launched_nodes: None,
            log_lines: Vec::new(),
            log_filter_node: String::new(),
            log_filter_text: String::new(),
            log_receiver: None,
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
        loop {
            match receiver.try_recv() {
                Ok(entry) => {
                    self.log_lines.push(UiLogLine {
                        node_id: entry.node_id,
                        line: entry.line,
                    });
                    if self.log_lines.len() > 500 {
                        let overflow = self.log_lines.len() - 500;
                        self.log_lines.drain(0..overflow);
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.log_receiver = None;
                    break;
                }
            }
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
            self.push_ui_log(format!(
                "desktop_control heal from_peer={} to_peer={}",
                from, to
            ));
        }
    }

    fn send_clear(&mut self) {
        if self.with_nodes_mut(Launcher::broadcast_clear_command) {
            self.push_ui_log("desktop_control clear".to_string());
        }
    }

    fn send_shutdown(&mut self) {
        if self.with_nodes_mut(Launcher::broadcast_shutdown_command) {
            self.push_ui_log("desktop_control shutdown".to_string());
        }
        self.stop_cluster();
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
            });

            ui.separator();
            ui.label(format!("Status: {}", self.status_line));
            let running_nodes = self.launched_nodes.as_ref().map_or(0, |nodes| nodes.len());
            ui.label(format!("Running Nodes: {}", running_nodes));

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
