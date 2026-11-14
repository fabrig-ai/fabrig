//! Store census text and JSON from one walk (client/ctl/results.md, 1 and 2).
use super::Text;
use jig_client::contract as c;
use skein_json::writer::Encoder;

pub(super) fn text(census: Option<&c::StoreCensus>, out: &mut Text) {
    render(census, &mut Output::Text(out));
}
pub(super) fn encode(census: Option<&c::StoreCensus>, out: &mut Encoder) {
    render(census, &mut Output::Json(out));
}

enum Output<'a> {
    Text(&'a mut Text),
    Json(&'a mut Encoder),
}
fn render(census: Option<&c::StoreCensus>, out: &mut Output<'_>) {
    if let Output::Json(out) = out {
        out.object_start();
        out.key(b"census");
    }
    match census {
        None => match out {
            Output::Text(out) => out.put(b"no census yet\n"),
            Output::Json(out) => out.null(),
        },
        Some(census) => {
            header(census, out);
            // The deployment document supplies keyspaces in prefix order.
            for row in &census.keyspaces {
                keyspace(row, out);
            }
            if let Output::Json(out) = out {
                out.array_end();
                out.object_end();
            }
        }
    }
    if let Output::Json(out) = out {
        out.object_end();
    }
}
fn header(census: &c::StoreCensus, out: &mut Output<'_>) {
    match out {
        Output::Text(out) => {
            out.put(b"census at commit ");
            out.number(census.commit);
            out.put(b", writes since commit ");
            out.number(census.since);
            out.put(b"\n");
        }
        Output::Json(out) => {
            out.object_start();
            number(b"commit", census.commit, out);
            number(b"since", census.since, out);
            out.key(b"keyspaces");
            out.array_start();
        }
    }
}
fn keyspace(row: &c::CensusKeyspace, out: &mut Output<'_>) {
    match out {
        Output::Text(out) => {
            if row.prefix.is_empty() {
                out.put(b"-");
            } else {
                for byte in &row.prefix {
                    out.put(&[skein_lib::bytes::hex_digit(byte >> 4u32), skein_lib::bytes::hex_digit(byte & 15)]);
                }
            }
            out.put(b" ");
            out.put(if row.name.is_empty() { b"-" } else { row.name.as_bytes() });
            for (label, value) in [
                (b" rows ".as_slice(), row.rows),
                (b" keys ", row.keys),
                (b" values ", row.values),
                (b" puts ", row.puts),
                (b" erases ", row.erases),
                (b" written ", row.written),
            ] {
                out.put(label);
                out.number(value);
            }
            out.put(b" heads ");
            if row.heads.is_empty() {
                out.put(b"-");
            }
        }
        Output::Json(out) => {
            out.object_start();
            out.key(b"prefix");
            super::hex(&row.prefix, out);
            out.key(b"name");
            out.string(row.name.as_bytes());
            number(b"rows", row.rows, out);
            number(b"keys", row.keys, out);
            number(b"values", row.values, out);
            out.key(b"heads");
            out.array_start();
        }
    }
    for (index, head) in row.heads.as_slice().iter().enumerate() {
        match out {
            Output::Text(out) => {
                if index != 0 {
                    out.put(b",");
                }
                match head.head {
                    Some(head) => out.number(u64::from(head)),
                    None => out.put(b"-"),
                }
                out.put(b":");
                out.number(head.rows);
            }
            Output::Json(out) => {
                out.object_start();
                out.key(b"head");
                match head.head {
                    Some(head) => out.unsigned(u64::from(head)),
                    None => out.null(),
                }
                number(b"rows", head.rows, out);
                out.object_end();
            }
        }
    }
    match out {
        Output::Text(out) => out.put(b"\n"),
        Output::Json(out) => {
            out.array_end();
            number(b"puts", row.puts, out);
            number(b"erases", row.erases, out);
            number(b"written", row.written, out);
            out.object_end();
        }
    }
}
fn number(key: &[u8], value: u64, out: &mut Encoder) {
    out.key(key);
    out.unsigned(value);
}
