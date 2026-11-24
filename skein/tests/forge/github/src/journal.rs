//! Write-ahead intentions contain identifiers and times, never token values.
use crate::{Result, directory::Directory};
use skein_forge_conformance::write::{self, Secrets};
const NAME: &str = "made.txt";
#[derive(Clone)]
pub enum Made {
    Branch { repo: String, name: String },
    Pull { repo: String, head: String, base: String },
    Issue { repo: String, title: String },
    Rules { repo: String, name: String },
    Access { installation: u64, repository: u64, owner: String, add: bool },
    Mint { installation: u64, at: u64 },
}
impl Made {
    fn line(&self) -> String {
        match self {
            Self::Branch { repo, name } => format!("branch\t{repo}\t{name}"),
            Self::Pull { repo, head, base } => format!("pull\t{repo}\t{head}\t{base}"),
            Self::Issue { repo, title } => format!("issue\t{repo}\t{title}"),
            Self::Rules { repo, name } => format!("rules\t{repo}\t{name}"),
            Self::Access { installation, repository, owner, add } => {
                format!("access\t{installation}\t{repository}\t{owner}\t{add}")
            }
            Self::Mint { installation, at } => format!("mint\t{installation}\t{at}"),
        }
    }
}
pub struct Journal {
    pub pending: Vec<(usize, Made)>,
    next: usize,
    pub numbers: std::collections::BTreeMap<usize, u64>,
}
impl Journal {
    pub fn create(out: &mut Directory, live: bool, secrets: &Secrets) -> Result<Self> {
        out.create_dir_all("")?;
        let bytes = if live { b"github-conformance-v1\tlive\n".as_slice() } else { b"github-conformance-v1\tfake\n" };
        write::check("journal", bytes, b"", secrets)?;
        out.create_synced(NAME, bytes)?;
        Ok(Self { pending: vec![], next: 0, numbers: std::collections::BTreeMap::new() })
    }
    fn append(out: &mut Directory, line: &str, secrets: &Secrets) -> Result<()> {
        write::check("journal", line.as_bytes(), b"", secrets)?;
        out.append_synced(NAME, line)
    }
    pub fn before(&mut self, out: &mut Directory, made: Made, secrets: &Secrets) -> Result<usize> {
        let row = made.line();
        if row.contains(['\r', '\n']) {
            return Err("invalid journal field".into());
        }
        self.next += 1;
        Self::append(out, &format!("make\t{}\t{row}", self.next), secrets)?;
        self.pending.push((self.next, made));
        Ok(self.next)
    }
    pub(crate) fn created(&mut self, out: &mut Directory, id: usize, number: u64, secrets: &Secrets) -> Result<()> {
        Self::append(out, &format!("number\t{id}\t{number}"), secrets)?;
        self.numbers.insert(id, number);
        Ok(())
    }
    pub(crate) fn cancel(&mut self, out: &mut Directory, id: usize, secrets: &Secrets) -> Result<()> {
        Self::append(out, &format!("cancel\t{id}"), secrets)?;
        self.pending.retain(|(key, _)| *key != id);
        Ok(())
    }
    pub(crate) fn done(&mut self, out: &mut Directory, id: usize, secrets: &Secrets) -> Result<()> {
        Self::append(out, &format!("done\t{id}"), secrets)?;
        self.pending.retain(|(key, _)| *key != id);
        Ok(())
    }
    pub fn load(out: &mut Directory) -> Result<(Self, bool)> {
        let data = out.read_bounded(NAME, 1_048_577)?;
        if data.len() > 1_048_576 {
            return Err("journal exceeds its bound".into());
        }
        // Only complete, synced intentions can have caused an external change.
        let complete = data.iter().rposition(|byte| *byte == b'\n').ok_or("journal header incomplete")? + 1;
        if complete < data.len() {
            out.truncate_synced(NAME, u64::try_from(complete)?)?;
        }
        let text = std::str::from_utf8(&data[..complete])?;
        let mut lines = text.lines();
        let live = match lines.next() {
            Some("github-conformance-v1\tlive") => true,
            Some("github-conformance-v1\tfake") => false,
            _ => return Err("unknown journal format".into()),
        };
        let mut result = Self { pending: vec![], next: 0, numbers: std::collections::BTreeMap::new() };
        for line in lines {
            let parts: Vec<_> = line.split('\t').collect();
            match parts.as_slice() {
                ["number", id, number] => {
                    let id: usize = id.parse()?;
                    if !result.pending.iter().any(|(key, _)| *key == id) {
                        return Err("number has no pending intention".into());
                    }
                    result.numbers.insert(id, number.parse()?);
                }
                ["done" | "cancel", id] => {
                    let id: usize = id.parse()?;
                    result.pending.retain(|(key, _)| *key != id);
                }
                ["make", id, tail @ ..] => {
                    let id: usize = id.parse()?;
                    if id <= result.next {
                        return Err("journal order invalid".into());
                    }
                    result.next = id;
                    let made = match tail {
                        ["branch", repo, name] => Made::Branch { repo: (*repo).into(), name: (*name).into() },
                        ["pull", repo, head, base] => {
                            Made::Pull { repo: (*repo).into(), head: (*head).into(), base: (*base).into() }
                        }
                        ["issue", repo, title] => Made::Issue { repo: (*repo).into(), title: (*title).into() },
                        ["rules", repo, name] => Made::Rules { repo: (*repo).into(), name: (*name).into() },
                        ["access", i, r, o, add] => Made::Access {
                            installation: i.parse()?,
                            repository: r.parse()?,
                            owner: (*o).into(),
                            add: add.parse()?,
                        },
                        ["mint", i, at] => Made::Mint { installation: i.parse()?, at: at.parse()? },
                        _ => return Err("journal intention invalid".into()),
                    };
                    result.pending.push((id, made));
                }
                _ => return Err("journal record invalid".into()),
            }
        }
        Ok((result, live))
    }
}
