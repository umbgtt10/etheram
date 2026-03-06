// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::process::Command;

#[test]
fn startup_without_config_path_returns_error() {
    // Arrange
    let mut command = Command::new(env!("CARGO_BIN_EXE_etheram-desktop"));

    // Act
    let output = command.output().expect("failed to run etheram-desktop");

    // Assert
    assert!(!output.status.success());
}
