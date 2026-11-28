//! A task's next-run wait, separate from its creator preference and its committed account
//! (client/domain/contract/documents.md, 1; domain/accounts-uses.md, 1.3).

pub use jig_domain_contract::ops::{TaskRunWait, TaskSourceState, TaskWaitRefused, TaskWaitWhy};
