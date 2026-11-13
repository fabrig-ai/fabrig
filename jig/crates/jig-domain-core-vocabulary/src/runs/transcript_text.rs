//! Loaded transcript rows (jig/docs/design/domain/engine/executors.md, 2).

use crate::TurnRecord;
use alloc::boxed::Box;

/// A loaded stored turn and its protocol-rendered UTF-8 text; never stored together.
pub type TranscriptRow = (TurnRecord, Option<Box<[u8]>>);
