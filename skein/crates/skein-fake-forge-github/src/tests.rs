mod app;
mod app_facts;
mod checks;
pub(crate) mod fixtures;
mod git;
mod inline;
mod installations;
mod job_logs;
mod keys;
mod pages_conversion;
mod pages_install;
mod pages_manifest;
mod service_app;
mod service_limits;
mod service_reads;

use crate::{
    app::App,
    installations::{Installations, Kind, Limits, Reach},
};
use skein_lib::Wall;

fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(1_000_000_000).expect("fixture time"))
}
fn app() -> App {
    crate::fixture::app()
}
fn store(seed: u64) -> Installations {
    let mut rows = Installations::new(&Limits { installations: 2, tokens: 2, listed: 2 }, seed);
    assert_eq!(rows.install(1, b"person", Kind::User, Reach::All, wall(1000)), Ok(1));
    rows
}

mod issues;
mod read_support;

mod people;
mod pulls;
mod reviews;

mod queries;

mod queue;
mod write_issues;
mod write_support;

mod write_branches;
mod write_pulls;

mod write_merge;

mod protection;

mod auth;

mod byte_faults;
mod deliveries;
mod delivery_http;

mod rates;

mod mutations;

mod comment_edit;

mod image;
