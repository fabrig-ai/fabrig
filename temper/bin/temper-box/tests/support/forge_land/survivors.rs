//! The box's descendants that outlive it, named for the live log (the plan ui-box-children).
//! Process groups and exits follow skein/docs/design/runtime/io/processes.md.

use std::fmt::Write;

/// One process as `/proc` shows it.
pub struct Row {
    pub pid: u32,
    pub parent: u32,
    pub group: u32,
    pub session: u32,
    pub command: String,
}

/// The rows of `pids` still in `/proc`, in order; a pid gone has none.
pub fn rows(pids: &[u32]) -> Vec<Row> {
    pids.iter()
        .filter_map(|&pid| {
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
            let command = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
            parse(pid, &stat, &command)
        })
        .collect()
}

/// Read fields after `comm`'s last `)`; command-line NULs become spaces.
pub(super) fn parse(pid: u32, stat: &str, command: &[u8]) -> Option<Row> {
    let (_, fields) = stat.rsplit_once(')')?;
    let mut fields = fields.split_whitespace();
    let _state = fields.next()?;
    Some(Row {
        pid,
        parent: fields.next()?.parse().ok()?,
        group: fields.next()?.parse().ok()?,
        session: fields.next()?.parse().ok()?,
        command: String::from_utf8_lossy(command).replace('\0', " ").trim_end().to_owned(),
    })
}

/// Name each survivor, then the stop's rows for its parent and group leader.
/// A process gone from `/proc` is named as gone.
pub fn lines(left: &[u32], now: &[Row], at_stop: &[Row]) -> String {
    let mut output = String::new();
    for pid in left {
        if let Some(row) = now.iter().find(|row| row.pid == *pid) {
            write_row(&mut output, "SURVIVOR", row);
            for stopped in at_stop.iter().filter(|stopped| stopped.pid == row.parent || stopped.pid == row.group) {
                write_row(&mut output, "AT STOP", stopped);
            }
        } else {
            writeln!(output, "SURVIVOR pid={pid} gone").expect("write survivor to String");
        }
    }
    output
}

fn write_row(output: &mut String, prefix: &str, row: &Row) {
    writeln!(
        output,
        "{prefix} pid={} parent={} group={} session={} command={}",
        row.pid, row.parent, row.group, row.session, row.command
    )
    .expect("write process row to String");
}
