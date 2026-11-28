//! Connector news classes (jig/docs/design/domain/connectors.md, 2).

use crate::ops::CallKey;
use alloc::boxed::Box;
use skein_lib::Token;

/// Connector-classified news can be lowered by a task policy.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NewsClass {
    Wakes,
    Kept,
    Dropped,
}

/// Subscription notifications.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Notification: a topic subscription was accepted; no answer.
    Subscribed { request: Token, key: CallKey, subscription: u64, topic: u64 },
    /// Notification: a topic unsubscription was accepted; no answer.
    Unsubscribed { request: Token, key: CallKey, subscription: u64, topic: u64 },
    /// Notification: a topic subscription was refused; no answer.
    Refused { request: Token },
    /// Notification: a task's interest in a topic ended; no answer.
    Ended { task: u64, subscription: u64 },
}

/// What a topic connector says to the engine.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// Unasked: classified news for a subscription.
    News { task: u64, subscription: u64, topic: u64, class: NewsClass, words: Box<[u8]> },
}
