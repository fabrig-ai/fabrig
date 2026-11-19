//! Help and the reference from the file's rows (protocol/configuration/file.md, 7).
use crate::{
    File, Shape, SignIn, binary, render,
    rows::{self, Form, Row, Set},
};
use std::fmt::Write;

const RESTART: &str = "Every key is read at a start; changing any key needs a restart.\n";

#[must_use]
pub fn help(shape: Shape) -> Box<[u8]> {
    let mut out = format!("# {}: {}\n\n{RESTART}\n", binary(shape), crate::FILE);
    out.push_str("| Key | Default | What it says |\n|---|---|---|\n");
    for row in rows::ROWS {
        if row.set == Set::Public && row.shapes.contains(&shape) {
            writeln!(out, "| `{}` | {} | {} |", key(row), row.default, row.line).expect("string");
        }
    }
    out.push_str("\n## Example file\n\n```toml\n");
    out.push_str(std::str::from_utf8(&render(&example(shape), shape)).expect("rendered UTF-8"));
    out.push_str("```\n");
    out.into_bytes().into_boxed_slice()
}

fn example(shape: Shape) -> File {
    match shape {
        Shape::Standalone => File { listen: Some("127.0.0.1:7100".parse().expect("socket")), ..File::default() },
        Shape::EngineAlone => File {
            listen: Some("127.0.0.1:7100".parse().expect("socket")),
            public: Some("https://temper.example.com".into()),
            workers: Some("10.0.0.1:7200".parse().expect("socket")),
            owner: Some("sam@example.com".into()),
            sign_in: Some(SignIn {
                client: "1234.apps.googleusercontent.com".into(),
                secret: "secrets/google.secret".into(),
                members: Vec::new(),
                forges: Vec::new(),
            }),
            ..File::default()
        },
        Shape::WorkerAlone => {
            File { engine: Some("10.0.0.1:7200".parse().expect("socket")), name: Some("w1".into()), ..File::default() }
        }
    }
}

fn key(row: &Row) -> String {
    match row.form {
        Form::Table | Form::Environment => format!("[{}]", row.key),
        Form::Members | Form::Records => format!("[[{}]]", row.key),
        Form::Socket
        | Form::Origin
        | Form::Address
        | Form::Name
        | Form::Visible(_)
        | Form::DataPath
        | Form::AbsolutePath
        | Form::ConfigPath
        | Form::Login
        | Form::Forge
        | Form::Addresses
        | Form::Redirect
        | Form::Subject
        | Form::Endpoint
        | Form::Api
        | Form::Base
        | Form::Transport
        | Form::Services => row.key.into(),
    }
}

#[must_use]
pub fn reference() -> Box<[u8]> {
    let mut out = format!(
        "# Configuration\n\nKind: reference\n\nThe keys of `{}`, their shapes, bounds and defaults.\n\n{RESTART}\n",
        crate::FILE
    );
    out.push_str("S: `temper-box`; E: `temper-engine`; W: `temper-worker`. Help lists public keys alone.\n\n## Keys\n\n| Key | Shapes | Set | Form and bound | Default | What it says |\n|---|---|---|---|---|---|\n");
    for row in rows::ROWS {
        let shapes = row
            .shapes
            .iter()
            .map(|shape| match shape {
                Shape::Standalone => "S",
                Shape::EngineAlone => "E",
                Shape::WorkerAlone => "W",
            })
            .collect::<Vec<_>>()
            .join(" ");
        let set = match row.set {
            Set::Public => "public",
            Set::Internal => "internal",
            Set::Test => "test",
        };
        writeln!(out, "| `{}` | {shapes} | {set} | {} | {} | {} |", key(row), row.form.words(), row.default, row.line)
            .expect("string");
    }
    writeln!(out, "\n## Where each member of {} went\n", crate::FORMER).expect("string");
    for row in rows::ROWS {
        for member in row.members {
            // A table and its leaf can give the same member; name the leaf once.
            if !rows::ROWS.iter().any(|other| other.key.len() > row.key.len() && other.members.contains(member)) {
                writeln!(out, "- `{member}` → `{}`.", key(row)).expect("string");
            }
        }
    }
    for member in temper_service_core::startup::moved::MEMBERS {
        writeln!(out, "- `{}`: {}.", member.path, member.words).expect("string");
    }
    for (path, name) in [("workers[].slots", "slots"), ("workers[].secret", "secret"), ("workers[].agent", "agent")] {
        let (_, words) = rows::MOVED.iter().find(|(key, _)| *key == name).expect("moved key");
        writeln!(out, "- `{path}`: {words}.").expect("string");
    }
    crate::merge::command(&mut out);
    out.into_bytes().into_boxed_slice()
}

#[cfg(test)]
mod tests;
