// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildError {
    MissingComponent(&'static str),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::MissingComponent(name) => write!(f, "Missing required component: {}", name),
        }
    }
}
