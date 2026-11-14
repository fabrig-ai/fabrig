//! Client versions are held to layouts (jig/docs/design/protocol/client/README.md, section 6).
//! Only texts cross the checks; the tree test reads main at the patch check's merge base.

use std::{
    fmt::{self, Write as _},
    fs,
    path::Path,
    process::Command,
};

use skein_codec::schema::{Schema, parse};
use skein_scratch::paths;

struct Wire {
    name: &'static str,
    owner: &'static str,
    schemas: &'static [&'static str],
}

const WIRES: &[Wire] = &[
    Wire {
        name: "jig-client-wire",
        owner: "jig/crates/jig-client-protocol-wire",
        schemas: &["client-v22.schema", "client-v22-part1.schema"],
    },
    Wire { name: "jig-public-wire", owner: "jig/crates/jig-client-protocol-wire", schemas: &["public-v1.schema"] },
    Wire {
        name: "temper-client-wire",
        owner: "temper/crates/temper-client-protocol-wire",
        schemas: &["client-wire-v1.schema"],
    },
    Wire {
        name: "temper-public-wire",
        owner: "temper/crates/temper-client-protocol-wire",
        schemas: &["public-v1.schema"],
    },
];

#[derive(Debug)]
struct Finding {
    wire: String,
    version: Option<u16>,
    wrong: String,
}

impl Finding {
    fn new(wire: &str, version: Option<u16>, wrong: String) -> Self {
        Self { wire: wire.to_owned(), version, wrong }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}", self.wire)?;
        if let Some(version) = self.version {
            write!(out, " {version}")?;
        }
        write!(out, ": {}", self.wrong)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Layout<'a> {
    version: u16,
    digest: &'a str,
}

fn layouts<'a>(wire: &str, text: &'a str) -> Result<Vec<Layout<'a>>, Vec<Finding>> {
    let mut lines = Vec::new();
    let mut findings = Vec::new();
    if !text.ends_with('\n') {
        findings.push(Finding::new(wire, None, "the list must end with a newline".into()));
    }
    for (at, line) in text.split_terminator('\n').enumerate() {
        if line.starts_with('#') {
            continue;
        }
        let pair = line.split_once(' ').and_then(|(version, digest)| {
            if version.is_empty()
                || !version.bytes().all(|byte| byte.is_ascii_digit())
                || digest.len() != 64
                || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return None;
            }
            Some(Layout { version: version.parse().ok()?, digest })
        });
        match pair {
            Some(layout) => lines.push(layout),
            None => findings.push(Finding::new(
                wire,
                line.split(' ').next().and_then(|word| word.parse().ok()),
                format!("line {} is malformed; expected DECIMAL HEX64 (lowercase)", at + 1),
            )),
        }
    }
    if lines.is_empty() {
        findings.push(Finding::new(wire, None, "the list has no layouts".into()));
    }
    if findings.is_empty() { Ok(lines) } else { Err(findings) }
}

fn fingerprint(schema: &Schema) -> String {
    let mut text = String::with_capacity(64);
    for byte in schema.fingerprints().family().digest() {
        write!(text, "{byte:02x}").expect("a String takes the digest");
    }
    text
}

fn now(wire: &str, schema: &str, list: &str) -> Vec<Finding> {
    let schema = match parse(schema.as_bytes()) {
        Ok(schema) => schema,
        Err(why) => return vec![Finding::new(wire, None, format!("schema is invalid: {why:?}"))],
    };
    let mut findings = Vec::new();
    let version = schema.version();
    if schema.spelled(schema.family()) != wire.as_bytes() {
        findings.push(Finding::new(wire, Some(version), "schema names another family".into()));
    }
    let lines = match layouts(wire, list) {
        Ok(lines) => lines,
        Err(wrong) => return wrong,
    };
    for pair in lines.windows(2) {
        let [before, after] = pair else { unreachable!("windows of two") };
        if before.version.checked_add(1) != Some(after.version) {
            findings.push(Finding::new(
                wire,
                Some(after.version),
                format!("versions must rise by one after {}", before.version),
            ));
        }
    }
    if let Some(last) = lines.last() {
        if last.version != version {
            findings.push(Finding::new(
                wire,
                Some(last.version),
                format!("last line's version is not the schema's family version {version}"),
            ));
        }
        let digest = fingerprint(&schema);
        if last.digest != digest {
            findings.push(Finding::new(
                wire,
                Some(version),
                format!("the schema's layout is not its line's; expected {version} {digest}"),
            ));
        }
    }
    findings
}

fn landed(wire: &str, main: Option<&str>, list: &str) -> Vec<Finding> {
    let Some(main) = main else { return Vec::new() };
    let mut branch = list.split_terminator('\n');
    for (at, line) in main.split_terminator('\n').enumerate() {
        if branch.next() != Some(line) {
            return vec![Finding::new(
                wire,
                line.split(' ').next().and_then(|word| word.parse().ok()),
                format!("landed line {} was edited or dropped; main's list must stand unchanged at the head", at + 1),
            )];
        }
    }
    Vec::new()
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|why| format!("git {}: {why}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!("git {}: {}: {}", args.join(" "), output.status, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output.stdout)
}

fn base(root: &Path) -> String {
    let args = if git(root, &["rev-parse", "--verify", "origin/main^{commit}"]).is_ok() {
        vec!["merge-base", "HEAD", "origin/main"]
    } else {
        vec!["rev-parse", "HEAD"]
    };
    let reference = String::from_utf8(git(root, &args).expect("read the landed base")).expect("git's base is UTF-8");
    let reference = reference.trim().to_owned();
    eprintln!("layouts check base: {reference} (git {})", args.join(" "));
    reference
}

fn main_list(root: &Path, base: &str, path: &str) -> Result<Option<String>, String> {
    let found = git(root, &["ls-tree", "--name-only", base, "--", path])?;
    if found.is_empty() {
        return Ok(None);
    }
    let object = format!("{base}:{path}");
    String::from_utf8(git(root, &["show", &object])?).map(Some).map_err(|why| format!("{path}: {why}"))
}

#[test]
fn client_wire_layouts_are_current_and_landed() {
    let root = paths::workspace();
    let base = base(&root);
    let mut failures = Vec::new();
    for wire in WIRES {
        let result = (|| {
            let mut schema = String::new();
            for part in wire.schemas {
                schema.push_str(
                    &fs::read_to_string(root.join(wire.owner).join("schema").join(part))
                        .map_err(|why| format!("{part}: {why}"))?,
                );
            }
            let path = format!("{}/schema/{}.layouts", wire.owner, wire.name);
            let list = fs::read_to_string(root.join(&path)).map_err(|why| format!("{path}: {why}"))?;
            let main = main_list(&root, &base, &path)?;
            let mut findings = now(wire.name, &schema, &list);
            findings.extend(landed(wire.name, main.as_deref(), &list));
            Ok::<_, String>(findings)
        })();
        match result {
            Ok(findings) => failures.extend(findings.iter().map(ToString::to_string)),
            Err(why) => failures.push(format!("{} (base {base}): {why}", wire.name)),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const WIRE: &str = "jig-client-wire";
const SCHEMA: &str = "family jig-client-wire 20\nenum EndedBy { result, requester }\n";

fn line(source: &str) -> String {
    let schema = parse(source.as_bytes()).expect("fixture schema");
    format!("{} {}\n", schema.version(), fingerprint(&schema))
}

#[test]
fn an_appended_variant_without_a_new_layout_is_refused() {
    let list = line(SCHEMA);
    assert!(now(WIRE, SCHEMA, &list).is_empty());
    let changed = SCHEMA.replace("result, requester", "result, requester, standing");
    let findings = now(WIRE, &changed, &list);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings
            .first()
            .expect("layout finding")
            .to_string()
            .starts_with("jig-client-wire 20: the schema's layout is not its line's"),
        "{findings:?}"
    );
}

#[test]
fn versions_rise_by_one() {
    let schema = SCHEMA.replace("20\n", "22\n");
    let current = line(&schema);
    let previous = current.replace("22 ", "21 ");
    assert!(now(WIRE, &schema, &format!("{previous}{current}")).is_empty());
    for version in [20, 22, 24] {
        let list = format!("{}{}", current.replace("22 ", &format!("{version} ")), current);
        let findings = now(WIRE, &schema, &list);
        assert_eq!(findings.len(), 1, "{list}: {findings:?}");
        assert!(
            findings
                .first()
                .expect("version finding")
                .to_string()
                .starts_with("jig-client-wire 22: versions must rise by one"),
            "{findings:?}"
        );
    }
}

#[test]
fn last_version_is_the_family_version() {
    let list = line(SCHEMA).replace("20 ", "19 ");
    let findings = now(WIRE, SCHEMA, &list);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings.first().expect("last version finding").to_string(),
        "jig-client-wire 19: last line's version is not the schema's family version 20"
    );
}

#[test]
fn malformed_lines_are_refused() {
    let valid = line(SCHEMA);
    assert!(layouts(WIRE, &format!("# Layouts\n{valid}")).is_ok());
    let digest = "a".repeat(64);
    for bad in [
        format!("20 {}\n", "a".repeat(63)),
        format!("20 {}\n", "a".repeat(65)),
        format!("20 {}\n", "A".repeat(64)),
        format!("20 {}\n", "g".repeat(64)),
        format!("+20 {digest}\n"),
        format!("65536 {digest}\n"),
        format!("20  {digest}\n"),
        format!("20\t{digest}\n"),
        format!("20 {digest} extra\n"),
        format!("20 {digest}\r\n"),
        format!("20 {digest}"),
        format!("{valid}\n"),
        "# Only a comment\n".into(),
        String::new(),
    ] {
        let findings = layouts(WIRE, &bad).expect_err(&format!("accepted malformed list {bad:?}"));
        assert!(!findings.is_empty());
        assert!(findings.iter().all(|finding| finding.wire == WIRE), "{findings:?}");
    }
}

#[test]
fn landed_lines_cannot_be_edited_or_dropped() {
    let first = line(SCHEMA);
    let next = first.replace("20 ", "21 ");
    let main = format!("# Layouts\n{first}{next}");
    for branch in [
        main.replace("20 ", "19 "),
        format!("# Layouts\n{next}"),
        format!("# Layouts\n{first}"),
        main.replace("# Layouts", "# Changed"),
    ] {
        let findings = landed(WIRE, Some(&main), &branch);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings.first().expect("landed finding").to_string().contains("was edited or dropped"),
            "{findings:?}"
        );
    }
    let edited_digest = main.replace(&fingerprint(&parse(SCHEMA.as_bytes()).expect("schema")), &"0".repeat(64));
    assert_eq!(landed(WIRE, Some(&main), &edited_digest).len(), 1);
    assert!(landed(WIRE, None, &main).is_empty());
    assert!(landed(WIRE, Some(&main), &main).is_empty());
    assert!(landed(WIRE, Some(&main), &format!("{main}{}", first.replace("20 ", "22 "))).is_empty());
}
