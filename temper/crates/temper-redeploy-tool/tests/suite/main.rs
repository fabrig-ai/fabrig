//! The redeploy's focused and seeded suites (redeploy.md, 8).
mod composition;
mod description;
mod fake;
mod fuzzy;
mod hosts;
mod record;
mod referee;
mod reference;
mod stage;

use temper_redeploy_tool::{
    record::{Journal, Record},
    redeploy::Mode,
};
fn journal() -> Journal {
    Journal::new(Record::new(temper_redeploy_tool::fake::NEW, Mode::Engine), None, Box::new(std::io::sink()))
}
