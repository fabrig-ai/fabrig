//! Integration tests collected in one suite.

mod cancel;
mod cuts;
mod digests;
mod exchange;
mod files;
mod invariants;
mod processes;
mod sockets;
mod time;
mod trace;

mod fuzzy {
    mod exchange;
    mod files;
}
