//! Service forge-table handoffs (temper/docs/design/draft/playground-reset.md, 3 and 4).
//! Record updates and box removal are stubs for their owning bodies.
use crate::engine::Engine;
use temper_engine_domain_forge::Forge;

pub(crate) fn kept(_engine: &mut Engine, _forge: Forge) {}

pub(crate) fn removed(_engine: &mut Engine, _forge: u16) {}

pub(crate) fn box_remove(_engine: &mut Engine, _forge: u16) {}
