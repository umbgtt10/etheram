// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use etheram_node::builders::observer_builder::ObserverBuilder;
use etheram_node::observer::Observer;

pub fn build_observer() -> Result<Box<dyn Observer>, String> {
    ObserverBuilder::default()
        .build()
        .map_err(|error| format!("failed to build observer: {error:?}"))
}
