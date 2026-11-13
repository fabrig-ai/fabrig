//! The engine start and its pieces (temper/docs/design/protocol/services.md, 4.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

pub use temper_engine_service_component::*;
pub use temper_engine_service_component::{clients, forge, forges, store, system};
pub use temper_engine_service_startup::*;
pub use temper_engine_service_stock::stock;
pub use temper_engine_service_vocabulary::{
    ForgeRuntime, UnreadableRow, assignment, family, limits, render_for_task, render_for_task_page, server, settings,
    settings_note, tokens, tools, translate, values,
};
pub mod startup;
