//! The engine's command table (temper/docs/design/protocol/README.md, 4.2).
use temper_engine_shell::forge::{FETCH, FORGE_CHILD, PREPARE, REMOVE, RUN, SIGN_IN, SOURCE};
use temper_shell::{
    commands::{DIRECTORIES, HELP, Row, Table, Who},
    directories,
};

pub const ENGINE: Table = Table { binary: "temper-engine", rows: &ROWS };

const ROWS: [Row; 10] = [
    HELP,
    Row {
        words: &["serve"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES]"],
        line: "Run an engine alone's document until stopped; temper-worker serve runs a worker alone, and the box's temper serve runs an engine and its worker in one process.",
        arguments: &[DIRECTORIES],
        settings: &[&directories::SETTINGS],
        notes: "Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure. Refuses a worker alone's document, naming temper-worker serve, or an engine and its worker in one process, naming the box's temper serve.",
    },
    Row {
        words: &["versions"],
        who: Who::Deployment,
        forms: &[""],
        line: "Print this binary's patch levels, its store's and families' versions, and its wires, one line each.",
        arguments: &[],
        settings: &[],
        notes: "Reads no configuration and no store.",
    },
    FETCH,
    PREPARE,
    RUN,
    SIGN_IN,
    SOURCE,
    REMOVE,
    FORGE_CHILD,
];
