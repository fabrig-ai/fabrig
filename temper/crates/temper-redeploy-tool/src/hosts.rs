//! The host and clock seam, POSIX words and processes (temper/docs/design/redeploy.md, 2.2, 7).
use crate::description::Description;
use std::{
    ffi::OsStr,
    fmt::Write as _,
    os::unix::ffi::OsStrExt,
    process::{Child, Command, Stdio},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    Engine,
    Worker(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
pub trait Hosts {
    fn run(&mut self, part: Part, line: &str) -> Output;
    fn run_each(&mut self, lines: &[(Part, String)]) -> Vec<Output>;
    fn now(&mut self) -> u64;
    fn sleep(&mut self, nanoseconds: u64);
}

#[must_use]
pub fn quote(word: &str) -> String {
    String::from_utf8(quote_bytes(word.as_bytes())).expect("quoting preserves UTF-8")
}
/// Byte form of the same POSIX word, including non-UTF-8 words. NUL cannot be an argv byte.
#[must_use]
pub fn quote_bytes(word: &[u8]) -> Vec<u8> {
    let mut quoted = vec![b'\''];
    for byte in word {
        if *byte == b'\'' {
            quoted.extend_from_slice(b"'\\''");
        } else {
            quoted.push(*byte);
        }
    }
    quoted.push(b'\'');
    quoted
}
#[must_use]
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut line = String::new();
    let mut rest = template;
    while let Some((before, after)) = rest.split_once('{') {
        line.push_str(before);
        let (name, tail) = after.split_once('}').expect("description checked the placeholder");
        let value = values.iter().find(|(key, _)| *key == name).expect("template's values are given").1;
        line.push_str(&quote(value));
        rest = tail;
    }
    line.push_str(rest);
    line
}
#[must_use]
pub fn words(words: &[&str]) -> String {
    words.iter().map(|word| quote(word)).collect::<Vec<_>>().join(" ")
}

/// The engine's ctl template, with JSON requested once and stdin closed (redeploy.md, 7).
pub fn ctl(description: &Description, arguments: &[&str], hosts: &mut dyn Hosts) -> Result<String, String> {
    let mut line = description.engine.ctl.clone();
    for word in arguments.iter().copied().filter(|word| *word != "--json") {
        write!(line, " {}", quote(word)).expect("writing a String");
    }
    line.push_str(" '--json' </dev/null");
    let answer = hosts.run(Part::Engine, &line);
    if answer.code == Some(0) {
        Ok(answer.stdout)
    } else {
        Err(format!("{}: exit {:?}: {}", arguments.join(" "), answer.code, answer.stderr.trim()))
    }
}

pub type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);
pub type Environment = [EnvironmentEntry];
pub struct Processes {
    engine: Vec<String>,
    workers: Vec<Vec<String>>,
    environment: Vec<EnvironmentEntry>,
}
impl Processes {
    #[must_use]
    pub fn new(description: &Description, environment: &Environment) -> Self {
        Self {
            engine: description.engine.run.clone(),
            workers: description.workers.iter().map(|worker| worker.run.clone()).collect(),
            environment: environment.to_vec(),
        }
    }
    fn spawn(&self, part: Part, line: &str) -> Result<Child, Output> {
        let prefix = match part {
            Part::Engine => &self.engine,
            Part::Worker(index) => &self.workers[index],
        };
        let Some((program, arguments)) = prefix.split_first() else {
            return Err(failed("empty run prefix"));
        };
        Command::new(program)
            .args(arguments)
            .arg(line)
            .env_clear()
            .envs(self.environment.iter().map(|(key, value)| (OsStr::from_bytes(key), OsStr::from_bytes(value))))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|why| failed(&format!("{}: {why}", prefix.join(" "))))
    }
}
fn failed(why: &str) -> Output {
    Output { code: None, stdout: String::new(), stderr: why.into() }
}
fn wait(child: Result<Child, Output>) -> Output {
    match child {
        Err(output) => output,
        Ok(child) => match child.wait_with_output() {
            Err(why) => failed(&format!("wait: {why}")),
            Ok(output) => {
                let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                if output.status.code().is_none() {
                    write!(stderr, "process ended: {}", output.status).expect("write to string");
                }
                Output {
                    code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr,
                }
            }
        },
    }
}
impl Hosts for Processes {
    fn run(&mut self, part: Part, line: &str) -> Output {
        wait(self.spawn(part, line))
    }
    fn run_each(&mut self, lines: &[(Part, String)]) -> Vec<Output> {
        let children: Vec<_> = lines.iter().map(|(part, line)| self.spawn(*part, line)).collect();
        children.into_iter().map(wait).collect()
    }
    #[expect(clippy::disallowed_types, reason = "redeploy.md, 7: the process Hosts supplies wall time at the seam")]
    fn now(&mut self) -> u64 {
        u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("wall time since epoch")
                .as_nanos(),
        )
        .expect("wall nanoseconds fit u64")
    }
    fn sleep(&mut self, nanoseconds: u64) {
        std::thread::sleep(Duration::from_nanos(nanoseconds));
    }
}
