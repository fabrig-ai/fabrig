//! Tests that replay (tools/tiers/layers.md, 6.1).
mod record;
mod redeploy;
mod referee;
use temper_redeploy_tool::{
    record::{Journal, Record},
    redeploy::Mode,
};
fn journal() -> Journal {
    Journal::new(Record::new(temper_redeploy_tool::fake::NEW, Mode::Engine), None, Box::new(std::io::sink()))
}
#[path = "../suite/description.rs"]
mod description;
#[path = "../suite/fake.rs"]
mod fake;
mod fuzzy;
#[path = "../suite/openings.rs"]
mod openings;
#[path = "../suite/reference.rs"]
mod reference;
#[path = "../suite/stage.rs"]
mod stage;
