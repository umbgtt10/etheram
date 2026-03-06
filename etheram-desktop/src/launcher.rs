// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_config::ClusterConfig;

pub struct Launcher;

impl Launcher {
    pub fn run(config: &ClusterConfig) -> Result<(), String> {
        config.validate()?;
        println!(
            "etheram-desktop bootstrap: fleet_nodes={}, validators={}",
            config.node.len(),
            config.fleet.validator_set.len()
        );
        Ok(())
    }
}
