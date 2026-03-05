// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(all(not(test), target_arch = "arm"))]
use embedded_alloc::LlffHeap;

#[cfg(all(not(test), target_arch = "arm"))]
#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

#[cfg(all(not(test), target_arch = "arm"))]
const HEAP_SIZE: usize = 524288;

pub fn init() {
    #[cfg(all(not(test), target_arch = "arm"))]
    unsafe {
        HEAP.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE);
    }
}
