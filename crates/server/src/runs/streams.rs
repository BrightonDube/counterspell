//! Per-user limit on open event streams (design Path 9 step 4, requirement R12.6).
//!
//! A stream holds a [`StreamGuard`]. The guard frees its slot when it is dropped, so a client that
//! disconnects abruptly, or a stream that ends for any reason, always gives its slot back.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
};

use counterspell_common::ids::UserId;

/// Counts the open streams of each user against one limit. Clones share the same counts.
///
/// The counts live in this process only. With one server process (the single-VPS deployment) that is
/// the whole picture; see design open question 13.
#[derive(Clone)]
pub struct StreamSlots {
    limit: u64,
    open: Arc<Mutex<HashMap<UserId, u64>>>,
}

/// One open stream. Dropping it frees the slot.
pub struct StreamGuard {
    owner: UserId,
    slots: StreamSlots,
}

impl StreamSlots {
    /// Creates a table that allows `limit` open streams per user.
    #[must_use]
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            open: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Takes a slot for `owner`, or returns `None` when that user already has `limit` open streams.
    #[must_use]
    pub fn try_acquire(&self, owner: UserId) -> Option<StreamGuard> {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let count = open.entry(owner).or_insert(0);
        if *count >= self.limit {
            return None;
        }
        *count += 1;
        Some(StreamGuard {
            owner,
            slots: self.clone(),
        })
    }

    /// Gives back one slot for `owner`. Called only by [`StreamGuard`]'s `Drop`.
    fn release(&self, owner: UserId) {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(count) = open.get_mut(&owner) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                open.remove(&owner);
            }
        }
    }
}

impl Drop for StreamGuard {
    fn drop(&mut self) {
        self.slots.release(self.owner);
    }
}

#[cfg(test)]
mod tests {
    use counterspell_common::ids::UserId;
    use uuid::Uuid;

    use super::StreamSlots;

    fn user(number: u128) -> UserId {
        UserId::from_uuid(Uuid::from_u128(number))
    }

    #[test]
    fn a_user_can_hold_no_more_streams_than_the_limit() {
        let slots = StreamSlots::new(2);
        let _first = slots.try_acquire(user(1)).expect("first slot");
        let _second = slots.try_acquire(user(1)).expect("second slot");

        assert!(slots.try_acquire(user(1)).is_none());
    }

    #[test]
    fn dropping_a_guard_frees_its_slot() {
        let slots = StreamSlots::new(1);
        let guard = slots.try_acquire(user(1)).expect("slot");
        assert!(slots.try_acquire(user(1)).is_none());

        drop(guard);

        assert!(slots.try_acquire(user(1)).is_some());
    }

    #[test]
    fn one_user_being_full_does_not_block_another_user() {
        let slots = StreamSlots::new(1);
        let _held = slots.try_acquire(user(1)).expect("slot for user 1");

        assert!(slots.try_acquire(user(2)).is_some());
    }
}
