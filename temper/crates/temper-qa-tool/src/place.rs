//! Setups restored and recipes proven by the place's own build (qa/places.md, 2, 4, 6–8).
mod files;
use crate::faces::{ArchiveFace, DriveFace, RecipeWalk, WalkOutput};
use crate::{Home, Ledger, Pass, Refusal, document as d, refused};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

/// Shared with the holder and the loop. Only the holder keeps drive.lock held.
#[derive(Clone, Debug)]
pub struct PlacePaths {
    pub directory: PathBuf,
    pub ports: Option<String>,
    pub bin: PathBuf,
    pub bundle: PathBuf,
    pub scratch: PathBuf,
    pub copies: PathBuf,
    pub setup: PathBuf,
    pub walk_log: PathBuf,
    pub drive_lock: PathBuf,
}
impl PlacePaths {
    #[must_use]
    pub fn new(directory: &Path) -> Self {
        Self {
            directory: directory.into(),
            ports: None,
            bin: directory.join("bin"),
            bundle: directory.join("bundle"),
            scratch: directory.join("box"),
            copies: directory.join("copies"),
            setup: directory.join("setup"),
            walk_log: directory.join("walk.log"),
            drive_lock: directory.join("drive.lock"),
        }
    }
    pub fn ensure(&self) -> Result<(), Refusal> {
        for path in [&self.bin, &self.bundle, &self.scratch, &self.copies] {
            io(path, fs::create_dir_all(path))?;
        }
        Ok(())
    }
}

/// Probe and release immediately: a holder left by an ended pass still owns its place.
pub fn place_busy(place: &PlacePaths) -> Result<bool, Refusal> {
    place.ensure()?;
    let file = io(
        &place.drive_lock,
        OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(&place.drive_lock),
    )?;
    match file.try_lock() {
        Ok(()) => Ok(false),
        Err(std::fs::TryLockError::WouldBlock) => Ok(true),
        Err(std::fs::TryLockError::Error(why)) => Err(refused(format!("{}: {why}", place.drive_lock.display()))),
    }
}

pub struct RecipeInput {
    pub after: String,
    pub lines: Vec<String>,
    pub readme: String,
    pub scripts: Option<Value>,
}
impl RecipeInput {
    pub fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let value: Value = serde_json::from_slice(bytes).map_err(|why| refused(format!("recipe refused: {why}")))?;
        let object = d::object(&value, &["after", "lines", "readme", "scripts"])?;
        let scripts = object.get("scripts").cloned();
        if scripts.as_ref().is_some_and(|scripts| !scripts.is_object()) {
            return Err(refused("recipe refused: scripts must be an object"));
        }
        let lines = d::strings(d::required(object, "lines")?)?;
        if lines.iter().any(|line| line.is_empty() || line.contains(['\n', '\r'])) {
            return Err(refused("recipe refused: each line must be nonempty and span one line"));
        }
        let readme = d::text(object, "readme")?;
        if readme.trim().is_empty() {
            return Err(refused("recipe refused: readme is empty"));
        }
        Ok(Self { after: d::text(object, "after")?, lines, readme, scripts })
    }
}

/// The ledger is read under its lock, then released before any process runs.
pub struct PassPlace<D, A> {
    pub paths: PlacePaths,
    pub setups: PathBuf,
    pub pass: Pass,
    pub drive: D,
    pub archive: A,
    ledger: Ledger,
}
struct CatalogueEntry {
    parent: Option<String>,
    refusal: Option<String>,
}
struct Recipe {
    name: String,
    hash: String,
}
#[derive(Clone)]
struct Copy {
    path: PathBuf,
    hash: String,
    commit: String,
}
impl<D: DriveFace, A: ArchiveFace> PassPlace<D, A> {
    pub fn for_session(home: &Home, session: u64, drive: D, archive: A) -> Result<Self, Refusal> {
        let (ledger, pass) = home.read(|ledger| {
            let number = ledger.pass_command_session(session)?.pass.expect("pass session");
            let pass = ledger.passes.iter().find(|pass| pass.number == number).expect("session's pass").clone();
            Ok((ledger.clone(), pass))
        })?;
        let directory = home.configuration.place_directory(&home.directory, &pass.place)?;
        let directory =
            std::path::absolute(&directory).map_err(|why| refused(format!("{}: {why}", directory.display())))?;
        let setups = std::path::absolute(home.directory.join("passes").join(pass.number.to_string()).join("setups"))
            .map_err(|why| refused(format!("pass {} setups: {why}", pass.number)))?;
        let mut paths = PlacePaths::new(&directory);
        paths.ports = home.configuration.ports.get(&pass.place).cloned();
        let place = Self { paths, setups, pass, drive, archive, ledger };
        place.available()?;
        Ok(place)
    }
    fn available(&self) -> Result<(), Refusal> {
        if place_busy(&self.paths)? {
            return Err(refused(format!(
                "the drive session is open at {}; close it first: temper-qa pass close",
                self.pass.place
            )));
        }
        Ok(())
    }
    pub fn setup(&mut self, name: &str, output: &mut impl Write) -> Result<u8, Refusal> {
        self.available()?;
        if name == "fresh" {
            remove(&self.paths.setup)?;
            self.empty()?;
            self.write_setup("fresh\n")?;
            say(output, "setup fresh: an empty box")?;
            return Ok(0);
        }
        self.restore(name, output)
    }
    fn catalogue(&mut self) -> Result<BTreeMap<String, CatalogueEntry>, Refusal> {
        let text = self.drive.list(&self.paths, &self.setups)?;
        let mut catalogue = BTreeMap::new();
        for line in text.lines() {
            let (head, description) =
                line.split_once(": ").ok_or_else(|| refused(format!("recipe list: invalid line {line}")))?;
            let (name, parent) = if let Some((name, parent)) = head.split_once(" (after ") {
                let parent =
                    parent.strip_suffix(')').ok_or_else(|| refused(format!("recipe list: invalid parent {head}")))?;
                (name, Some(parent.to_owned()))
            } else {
                (head, None)
            };
            if !recipe_name(name) || parent.as_ref().is_some_and(|parent| !recipe_name(parent)) {
                return Err(refused(format!("recipe list: invalid name {head}")));
            }
            catalogue.insert(
                name.into(),
                CatalogueEntry { parent, refusal: description.strip_prefix("refused: ").map(str::to_owned) },
            );
        }
        Ok(catalogue)
    }
    fn check_name(&self, name: &str, catalogue: &BTreeMap<String, CatalogueEntry>) -> Result<(), Refusal> {
        if name != "fresh" && !catalogue.contains_key(name) {
            let names =
                std::iter::once("fresh").chain(catalogue.keys().map(String::as_str)).collect::<Vec<_>>().join(", ");
            return Err(refused(format!("setup {name} is not in pass {}'s setups; setups: {names}", self.pass.number)));
        }
        Ok(())
    }
    fn chain(&mut self, name: &str) -> Result<Vec<Recipe>, Refusal> {
        let catalogue = self.catalogue()?;
        self.check_name(name, &catalogue)?;
        let mut chain: Vec<Recipe> = Vec::new();
        let mut next = Some(name.to_owned());
        while let Some(name) = next.take() {
            if chain.len() >= 9 || chain.iter().any(|recipe| recipe.name == name) {
                return Err(refused(format!("recipe {name}: cycle or more than eight parents")));
            }
            let hash = self.drive.hash(&self.paths, &self.setups, &name)?;
            let entry = catalogue.get(&name).ok_or_else(|| refused(format!("recipe {name}: missing parent")))?;
            if let Some(why) = &entry.refusal {
                return Err(refused(format!("recipe {name}: {why}")));
            }
            next.clone_from(&entry.parent);
            chain.push(Recipe { name, hash });
        }
        chain.reverse();
        Ok(chain)
    }
    fn empty(&self) -> Result<(), Refusal> {
        for name in ["config", "data"] {
            remove(&self.paths.scratch.join(name))?;
        }
        Ok(())
    }
    fn write_setup(&self, text: &str) -> Result<(), Refusal> {
        let temporary = self.paths.directory.join("setup.tmp");
        io(&temporary, fs::write(&temporary, text))?;
        io(&self.paths.setup, fs::rename(&temporary, &self.paths.setup))
    }
    fn copies(&self, name: &str) -> Result<Vec<Copy>, Refusal> {
        let directory = self.paths.copies.join(name);
        if !directory.exists() {
            return Ok(Vec::new());
        }
        let mut copies = Vec::new();
        for entry in io(&directory, fs::read_dir(&directory))? {
            let entry = io(&directory, entry)?;
            let file = entry.file_name();
            let Some((hash, commit)) =
                file.to_str().and_then(|name| name.strip_suffix(".tar")).and_then(|name| name.split_once('-'))
            else {
                continue;
            };
            if hex(hash, &[64]) && hex(commit, &[40, 64]) {
                copies.push(Copy { path: entry.path(), hash: hash.into(), commit: commit.into() });
            }
        }
        copies.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(copies)
    }
    fn save_copy(&mut self, recipe: &Recipe, output: &mut impl Write) -> Result<(), Refusal> {
        let directory = self.paths.copies.join(&recipe.name);
        io(&directory, fs::create_dir_all(&directory))?;
        let temporary = directory.join("copy.tmp");
        remove(&temporary)?;
        self.archive.create(&self.paths.scratch, &temporary)?;
        let kept = directory.join(format!("{}-{}.tar", recipe.hash, self.pass.build));
        io(&kept, fs::rename(&temporary, &kept))?;
        for entry in io(&directory, fs::read_dir(&directory))? {
            let entry = io(&directory, entry)?;
            if entry.path() != kept {
                remove(&entry.path())?;
            }
        }
        say(output, &format!("copy {} saved: {} at {}", recipe.name, short(&recipe.hash), short(&self.pass.build)))
    }
    fn walk(&mut self, name: &str, from: Option<&str>, output: &mut impl Write) -> Result<WalkOutput, Refusal> {
        say(output, &format!("walking {name} from {}", from.unwrap_or("fresh")))?;
        let walked = self.drive.walk(&RecipeWalk { place: &self.paths, setups: &self.setups, name, from })?;
        say(output, &format!("walk {name} ended: {}", walked.code))?;
        Ok(walked)
    }
    fn stop(
        &self,
        name: &str,
        recipe: &str,
        walked: &WalkOutput,
        source: Option<(&str, &Copy)>,
        output: &mut impl Write,
    ) -> Result<u8, Refusal> {
        let code = match walked.code {
            3 | 4 => {
                say(output, &format!("setup {name}: the box failed walking {recipe}"))?;
                4
            }
            5 => {
                let stopped =
                    walked.tail.lines().rev().find_map(|line| line.strip_prefix("stopped at ")).unwrap_or(recipe);
                say(output, &format!("setup {name}: stopped at {stopped}"))?;
                5
            }
            _ => 1,
        };
        if !walked.tail.is_empty() {
            say(output, &walked.tail)?;
        }
        if code == 5 {
            let failed = walked
                .tail
                .lines()
                .rev()
                .find_map(|line| line.strip_prefix("stopped at "))
                .and_then(|line| line.split_once(':'))
                .map_or(recipe, |(name, _)| name);
            self.facts(if failed == "walk" { recipe } else { failed }, source, output)?;
        }
        Ok(code)
    }
    fn facts(&self, recipe: &str, source: Option<(&str, &Copy)>, output: &mut impl Write) -> Result<(), Refusal> {
        if let Some((name, copy)) = source {
            say(output, &format!("copy started from: {name} made at {}", copy.commit))?;
        } else {
            say(output, "copy started from: none")?;
        }
        let copy = self.copies(recipe)?.pop();
        let previous = if let Some(copy) = &copy {
            say(output, &format!("recipe {recipe} last walked on: {}", copy.commit))?;
            self.ledger
                .builds
                .iter()
                .find(|build| build.commit == copy.commit)
                .and_then(|build| build.holds.as_deref())
                .unwrap_or_default()
        } else {
            say(output, &format!("recipe {recipe} last walked on: no copy"))?;
            &[]
        };
        say(output, "landings since:")?;
        let current = self
            .ledger
            .builds
            .iter()
            .find(|build| build.commit == self.pass.build)
            .and_then(|build| build.holds.as_deref())
            .unwrap_or_default();
        for landing in &self.ledger.landings {
            if current.contains(&landing.number) && !previous.contains(&landing.number) {
                say(output, &format!("landing {}: goal {}: {}", landing.number, landing.goal, landing.title))?;
            }
        }
        Ok(())
    }
    fn restore(&mut self, name: &str, output: &mut impl Write) -> Result<u8, Refusal> {
        let chain = self.chain(name)?;
        let hash = chain.last().expect("nonempty chain").hash.clone();
        remove(&self.paths.setup)?;
        // The whole log is retained across retries and subsequent walks.
        io(&self.paths.walk_log, File::create(&self.paths.walk_log))?;
        let mut ceiling = chain.len();
        let mut saved = Vec::new();
        loop {
            let mut source = None;
            for (index, recipe) in chain.iter().enumerate().take(ceiling).rev() {
                if let Some(copy) = self.copies(&recipe.name)?.into_iter().find(|copy| copy.hash == recipe.hash) {
                    source = Some((index, copy));
                    break;
                }
            }
            self.empty()?;
            if let Some((_, copy)) = &source {
                self.archive.extract(&copy.path, &self.paths.scratch)?;
            }
            let start =
                source.as_ref().map_or(0, |(index, _)| if index + 1 == chain.len() { *index } else { index + 1 });
            let mut retry = false;
            for (index, recipe) in chain.iter().enumerate().skip(start) {
                let from = if index == start {
                    source.as_ref().map(|(index, _)| chain[*index].name.as_str())
                } else {
                    Some(chain[index - 1].name.as_str())
                };
                let walk_source = if index == start {
                    source.clone()
                } else {
                    let parent = index - 1;
                    self.copies(&chain[parent].name)?
                        .into_iter()
                        .find(|copy| copy.hash == chain[parent].hash)
                        .map(|copy| (parent, copy))
                };
                let walked = self.walk(&recipe.name, from, output)?;
                if walked.code == 3
                    && let Some((index, copy)) = &walk_source
                {
                    remove(&copy.path)?;
                    ceiling = *index;
                    retry = true;
                    break;
                }
                if walked.code != 0 {
                    print_saved(output, &saved)?;
                    let source = walk_source.as_ref().map(|(index, copy)| (chain[*index].name.as_str(), copy));
                    return self.stop(name, &recipe.name, &walked, source, output);
                }
                if source.as_ref().is_none_or(|(copy_index, _)| *copy_index != index) {
                    self.save_copy(recipe, &mut saved)?;
                }
            }
            if retry {
                continue;
            }
            self.write_setup(&format!("{name} {hash}\n"))?;
            let how = match &source {
                Some((index, copy)) if *index + 1 == chain.len() => {
                    format!("restored from its copy made at {}", short(&copy.commit))
                }
                Some((index, _)) => format!("made again from {}'s copy", chain[*index].name),
                None => "made from a fresh box".into(),
            };
            say(output, &format!("setup {name}: {how}"))?;
            print_saved(output, &saved)?;
            return Ok(0);
        }
    }
    pub fn recipe(
        &mut self,
        name: &str,
        input: &RecipeInput,
        build_setups: &[String],
        output: &mut impl Write,
    ) -> Result<u8, Refusal> {
        self.available()?;
        if !recipe_name(name) || name == "fresh" {
            return Err(refused(format!("recipe {name}: invalid recipe name")));
        }
        if build_setups.iter().any(|built| built == name) {
            return Err(refused(format!("recipe {name} is among the build's setups")));
        }
        let catalogue = self.catalogue()?;
        self.check_name(&input.after, &catalogue)?;
        if input.after == name {
            return Err(refused(format!("recipe {name}: cannot be its own parent")));
        }
        let directory = self.setups.join(name);
        remove(&self.paths.setup)?;
        remove(&directory)?;
        io(&directory, fs::create_dir_all(&directory))?;
        let result = self.write_recipe(name, input, output);
        if result.as_ref().is_err() || result.as_ref().is_ok_and(|code| *code != 0) {
            remove(&directory)?;
        }
        result
    }
    fn write_recipe(&mut self, name: &str, input: &RecipeInput, output: &mut impl Write) -> Result<u8, Refusal> {
        let directory = self.setups.join(name);
        let mut text = "recipe 1\n".to_owned();
        if input.after != "fresh" {
            text.push_str("after \"");
            text.push_str(&input.after);
            text.push_str("\"\n");
        }
        for line in &input.lines {
            text.push_str(line);
            text.push('\n');
        }
        io(&directory, fs::write(directory.join("recipe"), text))?;
        io(&directory, fs::write(directory.join("README.md"), format!("{}\n", input.readme)))?;
        if let Some(scripts) = &input.scripts {
            let bytes = serde_json::to_vec_pretty(scripts).map_err(|why| refused(format!("recipe {name}: {why}")))?;
            io(&directory, fs::write(directory.join("scripts.json"), bytes))?;
        }
        let source = self.setups.parent().expect("pass setups directory").join("files");
        files::copy(&input.lines, &source, &directory.join("files"))?;
        let hash = self.drive.hash(&self.paths, &self.setups, name)?;
        let from = if input.after == "fresh" {
            remove(&self.paths.setup)?;
            self.empty()?;
            io(&self.paths.walk_log, File::create(&self.paths.walk_log))?;
            None
        } else {
            let restored = self.restore(&input.after, output)?;
            if restored != 0 {
                return Ok(restored);
            }
            Some(input.after.as_str())
        };
        remove(&self.paths.setup)?;
        let source = from
            .map(|name| self.copies(name).map(|mut copies| copies.pop().map(|copy| (name, copy))))
            .transpose()?
            .flatten();
        let walked = self.walk(name, from, output)?;
        if walked.code != 0 {
            return self.stop(name, name, &walked, source.as_ref().map(|(name, copy)| (*name, copy)), output);
        }
        self.save_copy(&Recipe { name: name.into(), hash: hash.clone() }, output)?;
        self.write_setup(&format!("{name} {hash}\n"))?;
        say(output, &format!("recipe {name} walked from {}: {}", input.after, short(&hash)))?;
        Ok(0)
    }
}
fn recipe_name(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase())
        && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
fn hex(text: &str, lengths: &[usize]) -> bool {
    lengths.contains(&text.len()) && text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn short(text: &str) -> &str {
    text.get(..12).unwrap_or(text)
}
fn remove(path: &Path) -> Result<(), Refusal> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => io(path, fs::remove_dir_all(path)),
        Ok(_) => io(path, fs::remove_file(path)),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(refused(format!("{}: {why}", path.display()))),
    }
}
fn io<T>(path: &Path, result: std::io::Result<T>) -> Result<T, Refusal> {
    result.map_err(|why| refused(format!("{}: {why}", path.display())))
}
fn print_saved(output: &mut impl Write, saved: &[u8]) -> Result<(), Refusal> {
    output.write_all(saved).and_then(|()| output.flush()).map_err(|why| refused(format!("output: {why}")))
}
fn say(output: &mut impl Write, line: &str) -> Result<(), Refusal> {
    writeln!(output, "{line}").and_then(|()| output.flush()).map_err(|why| refused(format!("output: {why}")))
}
