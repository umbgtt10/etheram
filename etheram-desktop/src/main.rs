// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_desktop::cluster_config::ClusterConfig;
use etheram_desktop::launcher::Launcher;
use etheram_desktop::ui;
use std::env;
use std::path::Path;
use std::process::ExitCode;

fn run() -> Result<(), String> {
    let mut args = env::args();
    let _binary_name = args.next();
    let Some(config_path) = args.next() else {
        return Err("usage: etheram-desktop <cluster.toml>".to_string());
    };
    let config = ClusterConfig::load_from_path(Path::new(&config_path))?;
    Launcher::run(&config, &config_path)?;
    ui::print_startup_banner();
    Ok(())
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
