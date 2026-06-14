//! Routing table: maps physical device handles to virtual Xbox slots.

use serde::{Deserialize, Serialize};

/// Routing table held behind `Arc<RwLock<RoutingTable>>`.
///
/// `slots[0..3]` are the 4 active game slots (virtual Xbox controllers 1-4).
/// `bench` holds the one benched player's device handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingTable {
    pub slots: [Option<u64>; 4],
    pub bench: Option<u64>,
}

impl Default for RoutingTable {
    fn default() -> Self {
        Self {
            slots: [None; 4],
            bench: None,
        }
    }
}

impl RoutingTable {
    /// Swap two device handles anywhere in the table (slots ↔ slots, slot ↔ bench, etc.).
    /// If a handle is not found, this is a no-op.
    pub fn swap(&mut self, handle_a: u64, handle_b: u64) {
        // Collect all mutable positions as (location, value) pairs and mutate in place.
        // We iterate over slots first, then bench.

        let pos_a = self.find_position(handle_a);
        let pos_b = self.find_position(handle_b);

        match (pos_a, pos_b) {
            (Some(Position::Slot(i)), Some(Position::Slot(j))) => {
                self.slots.swap(i, j);
            }
            (Some(Position::Slot(i)), Some(Position::Bench)) => {
                let tmp = self.slots[i];
                self.slots[i] = self.bench;
                self.bench = tmp;
            }
            (Some(Position::Bench), Some(Position::Slot(j))) => {
                let tmp = self.bench;
                self.bench = self.slots[j];
                self.slots[j] = tmp;
            }
            (Some(Position::Bench), Some(Position::Bench)) => {
                // Same position — no-op
            }
            _ => {
                log::warn!(
                    "swap({handle_a:#x}, {handle_b:#x}): one or both handles not in routing table"
                );
            }
        }
    }

    /// Assign a device handle to a game slot (0-3).
    pub fn assign_slot(&mut self, slot: usize, handle: u64) {
        assert!(slot < 4, "slot index out of range");
        self.slots[slot] = Some(handle);
    }

    /// Assign a device handle to the bench.
    pub fn assign_bench(&mut self, handle: u64) {
        self.bench = Some(handle);
    }

    /// Remove all references to a device handle (called on disconnect).
    pub fn remove_handle(&mut self, handle: u64) {
        for slot in &mut self.slots {
            if *slot == Some(handle) {
                *slot = None;
            }
        }
        if self.bench == Some(handle) {
            self.bench = None;
        }
    }

    fn find_position(&self, handle: u64) -> Option<Position> {
        for (i, slot) in self.slots.iter().enumerate() {
            if *slot == Some(handle) {
                return Some(Position::Slot(i));
            }
        }
        if self.bench == Some(handle) {
            return Some(Position::Bench);
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
enum Position {
    Slot(usize),
    Bench,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_slot_to_bench() {
        let mut table = RoutingTable::default();
        table.assign_slot(0, 10);
        table.assign_slot(1, 20);
        table.assign_bench(50);

        table.swap(10, 50);

        assert_eq!(table.slots[0], Some(50));
        assert_eq!(table.bench, Some(10));
        assert_eq!(table.slots[1], Some(20));
    }

    #[test]
    fn swap_slot_to_slot() {
        let mut table = RoutingTable::default();
        table.assign_slot(0, 1);
        table.assign_slot(3, 4);

        table.swap(1, 4);

        assert_eq!(table.slots[0], Some(4));
        assert_eq!(table.slots[3], Some(1));
    }

    #[test]
    fn swap_missing_handle_is_noop() {
        let mut table = RoutingTable::default();
        table.assign_slot(0, 99);

        table.swap(99, 999); // 999 not in table

        assert_eq!(table.slots[0], Some(99)); // unchanged
    }
}
