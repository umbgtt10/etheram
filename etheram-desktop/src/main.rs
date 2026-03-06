// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_desktop::launcher::Launcher;
use etheram_desktop::ui::gui_app::run_gui;
use etheram_desktop::ui::startup_banner::print_startup_banner;
use etheram_node_process::cluster_config::ClusterConfig;
use std::env;
use std::path::Path;
use std::process::ExitCode;

fn run_cli(config_path: &str) -> Result<(), String> {
    let config = ClusterConfig::load_from_path(Path::new(config_path))?;
    Launcher::run(&config, config_path)?;
    print_startup_banner();
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = env::args();
    let _binary_name = args.next();
    let Some(first_arg) = args.next() else {
        return Err("usage: etheram-desktop <cluster.toml> | --gui [cluster.toml]".to_string());
    };

    if first_arg == "--gui" {
        return run_gui(args.next());
    }

    run_cli(&first_arg)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
