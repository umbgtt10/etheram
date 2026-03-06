// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::OnceLock;

fn signal() -> &'static AtomicBool {
    static SHUTDOWN_SIGNAL: OnceLock<AtomicBool> = OnceLock::new();
    SHUTDOWN_SIGNAL.get_or_init(|| AtomicBool::new(false))
}

pub fn request_shutdown() {
    signal().store(true, Ordering::SeqCst);
}

pub fn is_shutdown_requested() -> bool {
    signal().load(Ordering::SeqCst)
}

pub fn reset_shutdown() {
    signal().store(false, Ordering::SeqCst);
}
