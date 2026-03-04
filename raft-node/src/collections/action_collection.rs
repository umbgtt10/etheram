// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;
use etheram_core::collection::Collection;

#[derive(Debug, Clone)]
pub struct ActionCollection<T> {
    actions: Vec<T>,
}

impl<T> ActionCollection<T> {
    pub fn from_vec(actions: Vec<T>) -> Self {
        Self { actions }
    }

    pub fn into_inner(self) -> Vec<T> {
        self.actions
    }
}

impl<T> Collection for ActionCollection<T> {
    type Item = T;
    type Iter<'a>
        = alloc::slice::Iter<'a, T>
    where
        T: 'a;

    fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.actions.len()
    }

    fn push(&mut self, item: T) {
        self.actions.push(item);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.actions.get(index)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.actions.iter()
    }

    fn clear(&mut self) {
        self.actions.clear();
    }
}
