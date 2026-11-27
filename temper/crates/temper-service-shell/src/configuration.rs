//! Binary-shaped configuration through the file's table
//! (temper/docs/design/protocol/configuration/file.md, 6).
use temper_config_shell::{Keys, Shape};
use temper_shell::directories::Directories;

pub struct Read {
    pub file: temper_config_shell::File,
    pub document: Box<[u8]>,
    pub keys: Keys,
}

/// Refuse the configuration before opening data or cache.
pub fn read(directories: &Directories, shape: Shape) -> Result<Read, String> {
    read_with(temper_config_shell::read(directories, shape), || directories.open(skein_shell::effective_user()))
}

/// Open data and cache only after a successful configuration read.
pub fn read_with(
    read: Result<temper_config_shell::Read, temper_config_shell::Refusal>,
    open: impl FnOnce() -> Result<(), String>,
) -> Result<Read, String> {
    let read = report(read)?;
    open()?;
    Ok(Read::from(read))
}

/// Offline readers do not create or open data directories.
pub fn read_only(directories: &Directories, shape: Shape) -> Result<Read, String> {
    report(temper_config_shell::read(directories, shape)).map(Read::from)
}

/// Report notices for offline commands that need the file reader's full result.
pub fn report(
    read: Result<temper_config_shell::Read, temper_config_shell::Refusal>,
) -> Result<temper_config_shell::Read, String> {
    report_to(read, std::io::stderr().lock())
}

fn report_to(
    read: Result<temper_config_shell::Read, temper_config_shell::Refusal>,
    mut log: impl std::io::Write,
) -> Result<temper_config_shell::Read, String> {
    let read = read.map_err(|why| why.to_string())?;
    for notice in &read.notices {
        writeln!(log, "{notice}").map_err(|why| format!("configuration notices: {why}"))?;
    }
    Ok(read)
}

impl From<temper_config_shell::Read> for Read {
    fn from(temper_config_shell::Read { file, document, keys, .. }: temper_config_shell::Read) -> Self {
        Self { file, document, keys }
    }
}

/// Program-login prints file refusals one per line and other problems on one line.
#[must_use]
pub fn login_problems(command: &str, lines: &str) -> String {
    if lines.lines().any(|line| {
        line.starts_with(&format!("{}:", temper_config_shell::FILE))
            || line.starts_with(&format!("{}: moved:", temper_config_shell::FORMER))
    }) {
        crate::prefixed(command, lines)
    } else {
        crate::prefixed(command, &lines.lines().collect::<Vec<_>>().join("; "))
    }
}

impl Read {
    /// Programs are process inputs, not members of the start's document (file.md, 4).
    pub fn programs(&self) -> Result<temper_config_shell::Programs, String> {
        self.programs_with(temper_shell::lookup::is_executable)
    }

    /// Check the configured program paths using the supplied executable lookup.
    pub fn programs_with(
        &self,
        is_executable: impl Fn(&std::path::Path) -> bool,
    ) -> Result<temper_config_shell::Programs, String> {
        let programs = self.file.programs.clone();
        let mut problems = Vec::new();
        for (key, path) in [
            ("programs.git", &programs.git),
            ("programs.claude", &programs.claude),
            ("programs.forgejo", &programs.forgejo),
            ("programs.runner", &programs.runner),
        ] {
            let Some(path) = path else { continue };
            let words = if !path.is_absolute() {
                Some(format!("{}: must be an absolute path", path.display()))
            } else if !is_executable(path) {
                Some(format!("{}: must be an executable regular file", path.display()))
            } else {
                None
            };
            if let Some(words) = words {
                problems.push(self.keys.render_key(key, &words));
            }
        }
        if problems.is_empty() { Ok(programs) } else { Err(problems.join("\n")) }
    }

    /// The shells already render member paths and the start's words. Translate at the
    /// output boundary, leaving lines outside the configuration and their endings intact.
    #[must_use]
    pub fn problems(&self, lines: &str) -> String {
        let mut rendered = String::new();
        for line in lines.split_inclusive('\n') {
            let (line, ending) = line.strip_suffix('\n').map_or((line, ""), |line| (line, "\n"));
            match line.split_once(": ") {
                Some((member, words)) => rendered.push_str(&self.keys.render_member(member, words)),
                None => rendered.push_str(line),
            }
            rendered.push_str(ending);
        }
        rendered
    }

    pub fn prepare<T>(&self, prepare: impl FnOnce(&[u8]) -> Result<T, String>) -> Result<T, String> {
        prepare(&self.document).map_err(|why| self.problems(&why))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temper_config_shell::{FILE, Facts};

    #[test]
    fn configuration_notices_write_one_line_each_in_order() {
        let facts = Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: false };
        let mut read = temper_config_shell::parse(b"", Shape::Standalone, &facts).expect("fixture file");
        let first = format!("{FILE}:2: earlier: ignored; test release took it away");
        let second = format!("{FILE}:3: later: moved: fixture settings");
        read.notices = vec![first.clone(), second.clone()];
        let mut stderr = Vec::new();
        let read = report_to(Ok(read), &mut stderr).expect("notices reported");
        assert_eq!(stderr, format!("{first}\n{second}\n").into_bytes());
        assert_eq!(read.notices, [first, second]);
    }
}
