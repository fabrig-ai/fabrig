//! Private, bounded request files (smith/docs/design/protocol/agent.status.md).
//! Keeps a directory, file counter and bound; knows no credentials or environment.
//! `Dumps::new` configures the writer; the agent writes whole files before submissions.

use std::fmt::Write as FmtWrite;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::PathBuf;

use smith_agent_service::{Dump, Dumped, Model, Started};
use smith_protocol_program::Given;

/// Writes one directory up to the shared byte bound, with best-effort diagnostics.
#[derive(Debug)]
pub struct Dumps {
    directory: PathBuf,
    bound: u64,
    number: u64,
    stopped: bool,
}

impl Dumps {
    /// Configures capture without touching the filesystem.
    #[must_use]
    pub fn new(directory: PathBuf, bound: u64) -> Dumps {
        Dumps { directory, bound, number: 0, stopped: false }
    }

    pub(crate) fn write(&mut self, dump: Dump, errors: &mut dyn Write) {
        if self.stopped {
            return;
        }
        let number = self.number;
        let Some(next) = number.checked_add(1) else {
            self.stopped = true;
            self.log("file counter exhausted", errors);
            return;
        };
        self.number = next;
        let (what, bytes) = contents(dump.what);
        let identity = match dump.identity {
            Some(bytes) => {
                let mut value = String::with_capacity(32);
                for byte in bytes {
                    write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
                }
                value
            }
            None => "none".into(),
        };
        let name = format!("{identity}-a{:06}-{number:06}-{what}", dump.activation);
        if let Err(why) = self.store(&name, number, &bytes) {
            self.log(&why, errors);
        }
    }

    fn directory(&self) -> Result<(), String> {
        if crate::program::private_directory(&self.directory).map_err(|why| why.to_string())? {
            return Ok(());
        }
        match DirBuilder::new().recursive(true).mode(0o700).create(&self.directory) {
            Ok(()) => {}
            Err(why) if why.kind() == io::ErrorKind::AlreadyExists => {}
            Err(why) => return Err(why.to_string()),
        }
        if crate::program::private_directory(&self.directory).map_err(|why| why.to_string())? {
            Ok(())
        } else {
            Err("directory is absent after creation".into())
        }
    }

    fn store(&mut self, name: &str, number: u64, bytes: &[u8]) -> Result<(), String> {
        self.directory()?;
        let mut total = 0_u64;
        for entry in fs::read_dir(&self.directory).map_err(|why| why.to_string())? {
            let size = entry.and_then(|entry| entry.metadata()).map_err(|why| why.to_string())?.len();
            total = total.checked_add(size).ok_or("directory size overflow")?;
        }
        if total.checked_add(skein_lib::len_u64(bytes.len())).is_none_or(|size| size > self.bound) {
            self.stopped = true;
            return Err(format!("byte bound {} reached", self.bound));
        }
        let part = self.directory.join(format!(".dump-{}-{number}", std::process::id()));
        let mut file =
            OpenOptions::new().write(true).create_new(true).mode(0o600).open(&part).map_err(|why| why.to_string())?;
        let result = (|| {
            file.write_all(bytes)?;
            drop(file);
            fs::rename(&part, self.directory.join(name))
        })();
        if let Err(why) = result {
            drop(fs::remove_file(&part));
            return Err(why.to_string());
        }
        Ok(())
    }

    fn log(&self, why: &str, errors: &mut dyn Write) {
        let detail =
            format!("request dump: {}: {why}", self.directory.display()).replace('\r', "\\r").replace('\n', "\\n");
        crate::agent_shell::diagnostic(errors, &detail);
        // Refused directories stay untouched, including by the diagnostic.
        if crate::program::private_directory(&self.directory).ok() != Some(true) {
            return;
        }
        if let Ok(mut log) =
            OpenOptions::new().append(true).create(true).mode(0o600).open(self.directory.join("dump.log"))
        {
            crate::agent_shell::diagnostic(&mut log, &detail);
        }
    }
}

fn contents(what: Dumped) -> (&'static str, Box<[u8]>) {
    match what {
        Dumped::Started(started) => ("start.txt", start(started).into_bytes().into()),
        Dumped::Request { provider, body } => match provider {
            skein_llm::Provider::OpenAi => ("openai.json", body),
            skein_llm::Provider::Anthropic => ("anthropic.json", body),
        },
        Dumped::Program(given) => match given {
            Given::Arguments(arguments) => {
                let mut bytes = Vec::new();
                for argument in arguments {
                    bytes.extend_from_slice(&argument);
                    bytes.push(b'\n');
                }
                ("claude-code-argv.txt", bytes.into())
            }
            Given::Instructions(bytes) => ("claude-code-instructions.md", bytes),
            Given::Tools(bytes) => ("claude-code-tools.json", bytes),
            Given::Input(bytes) => ("claude-code-stdin.json", bytes),
            Given::Init(bytes) => ("claude-code-init.json", bytes),
        },
    }
}

fn model(output: &mut String, name: &str, model: &Model) {
    writeln!(output, "{name} model: {}", String::from_utf8_lossy(&model.name))
        .expect("writing to a String cannot fail");
    writeln!(output, "{name} effort: {}", model.effort.as_deref().map_or("default".into(), String::from_utf8_lossy))
        .expect("writing to a String cannot fail");
}

fn start(start: Started) -> String {
    let mode = match start.mode {
        smith_domain::run::charter::Mode::Finite => "finite",
        smith_domain::run::charter::Mode::Open => "open",
    };
    let mut output = format!(
        "mode: {mode}\nresume: {}\nwaiting: {} ns\nbudget turns: {}\nbudget spend: {}\nbudget time: {} ns\nreserve turns: {}\nreserve time: {} ns\n",
        start.resume,
        start.waiting.as_nanos(),
        start.budget.turns,
        start.budget.spend,
        start.budget.time.as_nanos(),
        start.reserve.turns,
        start.reserve.time.as_nanos(),
    );
    model(&mut output, "main", &start.main);
    for child in &start.models {
        model(&mut output, "sub-agent", child);
    }
    write!(
        output,
        "inspect: {}\nmodify: {}\nshell: {}\nagents: {}\nwait: {}\ndeliver: {}\n",
        start.tools.inspect, start.tools.modify, start.tools.shell, start.agents, start.wait, start.deliver
    )
    .expect("writing to a String cannot fail");
    for tool in start.host_tools {
        writeln!(output, "host tool: {}", String::from_utf8_lossy(&tool)).expect("writing to a String cannot fail");
    }
    for title in start.titles {
        writeln!(output, "brief title: {}", String::from_utf8_lossy(&title)).expect("writing to a String cannot fail");
    }
    output
}
