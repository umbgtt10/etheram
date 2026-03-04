// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(not(any(feature = "channel-transport", feature = "udp-transport")))]
compile_error!("Must select exactly one transport feature: 'channel-transport' or 'udp-transport'");

#[cfg(all(feature = "channel-transport", feature = "udp-transport"))]
compile_error!("Features 'channel-transport' and 'udp-transport' are mutually exclusive");

#[cfg(all(feature = "in-memory-storage", feature = "semihosting-storage"))]
compile_error!("Features 'in-memory-storage' and 'semihosting-storage' are mutually exclusive");

#[cfg(not(any(
    feature = "channel-external-interface",
    feature = "udp-external-interface"
)))]
compile_error!(
    "Must select exactly one external interface feature: 'channel-external-interface' or 'udp-external-interface'"
);

#[cfg(all(
    feature = "channel-external-interface",
    feature = "udp-external-interface"
))]
compile_error!(
    "Features 'channel-external-interface' and 'udp-external-interface' are mutually exclusive"
);

#[cfg(feature = "channel-transport")]
pub mod in_memory;

#[cfg(feature = "channel-transport")]
pub use in_memory::setup::init;
