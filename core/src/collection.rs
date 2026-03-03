// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::iter::Iterator;
use core::option::Option;

pub trait Collection {
    type Item;
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a,
        Self::Item: 'a;

    fn new() -> Self;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn push(&mut self, item: Self::Item);
    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn iter(&self) -> Self::Iter<'_>;
    fn clear(&mut self);
}
