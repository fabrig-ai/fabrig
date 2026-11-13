//! Configuration reference pages (temper/docs/design/protocol/documentation.md, 8).
use super::members::{self, Member, Shape};
use jig_protocol_config::{self as config, read};
use skein_json::{Document, document::Kind, writer::Encoder};
use skein_lib::{Decimal, List, Overflow, Writer};

/// The document's top level and application members, with values from init's filled document.
pub fn configuration_reference(init: &[u8], out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"# Configuration\n\nKind: reference\n\nThe configuration document's top level and application members, their bounds and what temper-box init --box writes.\n")?;
    render(init, members::shapes().as_slice(), members::members().as_slice(), false, out)?;
    merge_command(out)
}

/// The engine and worker members, with values from init's filled document.
pub fn engine_configuration_reference(init: &[u8], out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"# Engine configuration\n\nKind: reference\n\nThe engine configuration members and their bounds.\n")?;
    out.put(b"\nA seeded project's policy must be a number; inline policies are refused.\n")?;
    render(init, &config::members::SHAPES, &config::members::MEMBERS, true, out)
}

/// The merge command's contract (temper/docs/design/protocol/tooling.md, 4.1).
fn merge_command(out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"\n## The merge command\n\n`merge_job` names this command, a path in the repository being merged. It settles the conflicts its project's rules know, with no model, before an agent is asked. temper takes it from the merge in progress: a command file left in conflict, or absent from the merge, is its failure.\n\n- **What it is given:** its arguments, `merge` and the path of `request.json`, which holds `{\"version\":1,\"head\":\"HEAD\",\"base\":\"BASE\"}`: the head merged into, and the commit merged in. Its environment is `PATH` (`")?;
    out.put(crate::limits::TOOLING_PATH)?;
    out.put(b"`), `LANG` (`C.UTF-8`), and `HOME` and `TEMPER_JOB`, both the job's directory, which is its working directory too. No credential reaches it. It has ")?;
    out.put(
        Decimal::of(
            crate::limits::MERGE_DEADLINE
                .as_nanos()
                .checked_div(skein_lib::Duration::from_secs(1).as_nanos())
                .expect("one second has nonzero nanoseconds"),
        )
        .as_bytes(),
    )?;
    out.put(b" seconds.\n- **Where it works:** the repository is the directory of its own name under `TEMPER_JOB`, at the head with the base merged in and not committed; the conflicting files are marked and unmerged in the index (`git diff --name-only --diff-filter=U`).\n- **What it must do:** edit the conflicting files it knows and `git add` each it settles, without committing, aborting the merge or writing another branch; then write `result.json` in `TEMPER_JOB`, at most ")?;
    out.put(Decimal::of(u64::from(crate::limits::MERGE_RESULT_BYTES)).as_bytes())?;
    out.put(b" bytes. Where the paths do not fit, it lists those that do, in index order, and adds `\"more\":N`, the count left out. The same head and base must give the same tree.\n- **Exit 0:** `{\"settled\":[PATH]}`, no file left unmerged: temper commits the tree as the merge, with both parents, and pushes it. A file left unmerged or still marked fails the job.\n- **Exit 3:** `{\"settled\":[PATH],\"left\":[PATH]}`, the conflicts it could not settle: nothing is committed or pushed, and an agent resolves them in a merge made afresh, this result in its words.\n- **Any other exit,** or a result missing or past its bound, is the command's failure. A job that fails for good holds the change, as a resolution that fails does.\n")?;
    Ok(())
}

fn accepted(path: &str, engine: bool) -> bool {
    if path.starts_with("workers") || path.starts_with("application.workers[") {
        return false;
    }
    if engine {
        for stored in [
            "engine.charters",
            "engine.templates",
            "engine.policies",
            "engine.rules",
            "engine.projects[].policy",
            "engine.llm.models",
            "engine.llm.providers[].models",
        ] {
            if path == stored
                || path.starts_with(stored) && path.as_bytes().get(stored.len()) == Some(&b'.')
                || path.starts_with(stored) && path.as_bytes().get(stored.len()) == Some(&b'[')
            {
                return false;
            }
        }
    }
    true
}

fn render(init: &[u8], shapes: &[Shape], rows: &[Member], engine: bool, out: &mut Writer) -> Result<(), Overflow> {
    let bytes = u32::try_from(init.len()).expect("init document fits the configuration byte bound");
    let document = config::document(init, &config::DocumentLimits { bytes, tokens: bytes, depth: 32 })
        .expect("reference input is init's filled document");
    out.put(b"\nEvery member is read at startup; a change to any member needs a restart.\n\nObjects and lists of objects name their shapes below; init values show objects and list lengths, with scalar values written as JSON.\n")?;
    if !engine {
        out.put(b"Address objects use the engine page's address shape; see temper help configuration-engine. An engine alone runs no local worker.\n")?;
    }
    for shape in shapes {
        let mut heading = None;
        for path in &paths(shape, engine) {
            if accepted(path, engine) {
                heading = Some(*path);
                break;
            }
        }
        let Some(heading) = heading else { continue };
        out.put(b"\n## ")?;
        out.put(if heading.is_empty() { b"document" } else { heading.as_bytes() })?;
        out.put(b"\n\nShape: `")?;
        out.put(shape.name.as_bytes())?;
        out.put(b"`. Paths:")?;
        for path in &paths(shape, engine) {
            if accepted(path, engine) {
                out.put(b" `")?;
                out.put(if path.is_empty() { b"document" } else { path.as_bytes() })?;
                out.put(b"`")?;
            }
        }
        out.put(b". ")?;
        out.put(shape.line.as_bytes())?;
        out.put(b"\n\n")?;
        for row in rows {
            if row.shape == shape.name {
                render_row(&document, shape, row, engine, out)?;
            }
        }
    }
    Ok(())
}

fn render_row(
    document: &Document,
    shape: &Shape,
    row: &Member,
    engine: bool,
    out: &mut Writer,
) -> Result<(), Overflow> {
    if engine && row.name == "models" && (shape.name == "llm" || shape.name == "llm-provider") {
        return Ok(());
    }
    out.put(b"- `")?;
    out.put(row.name.as_bytes())?;
    out.put(b"`: ")?;
    if let Some(gone) = row.gone {
        out.put(b"refused: ")?;
        out.put(gone.as_bytes())?;
        return out.put(b"\n");
    }
    out.put(row.kind.as_bytes())?;
    if !row.bound.is_empty() {
        out.put(b", ")?;
        out.put(row.bound.as_bytes())?;
    }
    let required = row.required || engine && shape.name == "llm-provider" && row.name == "endpoint";
    out.put(if required { b"; required; " } else { b"; optional; " })?;
    let whole = row.name == "overrides" || row.kind == "address";
    let mut values = List::with_capacity(document.len());
    for path in &paths(shape, engine) {
        if accepted(path, engine) {
            for at in &at_path(document, path) {
                if let Some(at) = document.member(*at, row.name.as_bytes()) {
                    let mut duplicate = false;
                    for previous in &values {
                        duplicate |= same_value(document, *previous, at, whole);
                    }
                    if !duplicate {
                        values.push(at).expect("values bounded by the document");
                    }
                }
            }
        }
    }
    if values.is_empty() {
        out.put(b"init leaves it out. ")?;
    } else {
        out.put(b"init writes ")?;
        for (index, at) in values.iter().enumerate() {
            if index != 0 {
                out.put(b" | ")?;
            }
            value(document, *at, whole, out)?;
        }
        out.put(b". ")?;
    }
    out.put(row.line.as_bytes())?;
    out.put(b"\n")
}

fn at_path(document: &Document, path: &str) -> List<u32> {
    let mut ats = List::with_capacity(document.len());
    ats.push(0).expect("root");
    if path.is_empty() {
        return ats;
    }
    for part in path.split('.') {
        let mut next = List::with_capacity(document.len());
        let (name, array) = match part.strip_suffix("[]") {
            Some(name) => (name, true),
            None => (part, false),
        };
        for at in &ats {
            if let Some(at) = document.member(*at, name.as_bytes()) {
                if array {
                    if let Ok(items) = read::list(document, at, document.len()) {
                        for item in &items {
                            next.push(*item).expect("document indices");
                        }
                    }
                } else {
                    next.push(at).expect("document indices");
                }
            }
        }
        ats = next;
    }
    ats
}

fn same_value(document: &Document, left: u32, right: u32, whole: bool) -> bool {
    if !whole
        && document.token(left).expect("value").kind == Kind::ObjectStart
        && document.token(right).expect("value").kind == Kind::ObjectStart
    {
        return true;
    }
    let left_end = read::after(document, left).expect("JSON value");
    let right_end = read::after(document, right).expect("JSON value");
    if left_end.checked_sub(left) != right_end.checked_sub(right) {
        return false;
    }
    for (left, right) in (left..left_end).zip(right..right_end) {
        let left = document.token(left).expect("JSON token");
        let right = document.token(right).expect("JSON token");
        if left.kind != right.kind || document.text(left) != document.text(right) {
            return false;
        }
    }
    true
}

fn value(document: &Document, at: u32, whole: bool, out: &mut Writer) -> Result<(), Overflow> {
    let token = document.token(at).expect("JSON value");
    if !whole {
        match token.kind {
            Kind::ObjectStart => return out.put(b"object"),
            Kind::ArrayStart => {
                let items = read::list(document, at, document.len()).expect("JSON array");
                let mut objects = false;
                for at in &items {
                    objects |= document.token(*at).expect("array item").kind == Kind::ObjectStart;
                }
                if objects {
                    out.put(b"list (")?;
                    out.put(Decimal::of(u64::from(items.len())).as_bytes())?;
                    return out.put(if items.len() == 1 { b" entry)" } else { b" entries)" });
                }
            }
            Kind::ObjectEnd
            | Kind::ArrayEnd
            | Kind::Key
            | Kind::String
            | Kind::Number
            | Kind::True
            | Kind::False
            | Kind::Null
            | Kind::Long => {}
        }
    }
    let limits = skein_json::writer::Limits { depth: 32, length: u32::MAX };
    let end = read::after(document, at).expect("JSON value");
    let mut measure = Encoder::measure(&limits);
    json(document, at, end, &mut measure);
    let length = measure.measured().expect("init JSON value");
    let mut encoder = Encoder::write(length, &limits);
    json(document, at, end, &mut encoder);
    out.put(&encoder.finish())
}

fn json(document: &Document, at: u32, end: u32, out: &mut Encoder) {
    for index in at..end {
        let token = document.token(index).expect("JSON token");
        let text = document.text(token).expect("JSON text");
        match token.kind {
            Kind::ObjectStart => out.object_start(),
            Kind::ObjectEnd => out.object_end(),
            Kind::ArrayStart => out.array_start(),
            Kind::ArrayEnd => out.array_end(),
            Kind::Key => out.key(text),
            Kind::String => out.string(text),
            Kind::Number => out.number(text),
            Kind::True => out.boolean(true),
            Kind::False => out.boolean(false),
            Kind::Null => out.null(),
            Kind::Long => unreachable!("init document retains its text"),
        }
    }
}

fn paths(shape: &Shape, engine: bool) -> List<&'static str> {
    let mut paths = List::with_capacity(
        u32::try_from(shape.paths.len().checked_add(members::ADDRESS_PATHS.len()).expect("shape paths"))
            .expect("bounded paths"),
    );
    for path in shape.paths {
        paths.push(*path).expect("shape paths counted");
    }
    if engine && shape.name == "address" {
        for path in members::ADDRESS_PATHS {
            paths.push(*path).expect("shared address paths counted");
        }
    }
    paths
}
