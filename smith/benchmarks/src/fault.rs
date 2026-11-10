//! Harness failures retain their kind and exact detail, never persisted kind data.
//! Preparation labels follow the ends in smith/docs/design/benchmarks/README.md, 8.3.

use crate::isolate::SetupStep;

/// What kind of failure a harness step met; the fault's detail says which.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    /// A file, directory or process call failed.
    Io,
    /// A record, document or stream did not encode or decode.
    Format,
    /// An authored input, a pin, a configuration or an argument was refused.
    Input,
    /// What was read disagrees with what was pinned or written.
    Mismatch,
    /// A size, time or count bound was passed, or its arithmetic overflowed.
    Bound,
    /// A login is absent, lapsed or changed.
    Credential,
    /// An agent's event stream or its collection failed.
    Stream,
    /// A preparation step refused the attempt before inference.
    Setup(SetupStep),
}

/// One failure of the harness: its kind and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: FaultKind,
    pub detail: String,
}

impl Fault {
    /// Attach a kind without changing the failure's words.
    pub fn new(kind: FaultKind, detail: impl Into<String>) -> Self {
        Self { kind, detail: detail.into() }
    }

    /// Keep a document reader's exact file and field prefix without changing kind.
    pub(crate) fn document(self, file: &std::path::Path, key: &str) -> Self {
        Self { detail: crate::Refusal::new(file, key, &self.detail).to_string(), ..self }
    }

    /// Label a preparation call, preserving a more specific preparation step.
    pub fn at<E: Into<Self>>(step: SetupStep) -> impl FnOnce(E) -> Self {
        move |error| {
            let fault = error.into();
            match fault.kind {
                FaultKind::Setup(_) => fault,
                _ => Self { kind: FaultKind::Setup(step), ..fault },
            }
        }
    }
}

/// Refuse an authored condition with its typed detail.
pub(crate) fn refuse_if(refused: bool, kind: FaultKind, detail: &str) -> Result<(), Fault> {
    if refused { Err(Fault::new(kind, detail)) } else { Ok(()) }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for Fault {}

macro_rules! kind {
    ($kind:expr, $name:ident; $($error:ty),* $(,)?) => {
        pub(crate) fn $name(detail: impl Into<String>) -> Fault {
            Fault::new($kind, detail)
        }
        $(impl From<$error> for Fault {
            fn from(error: $error) -> Self { $name(error.to_string()) }
        })*
    };
}

kind!(FaultKind::Io, io; std::io::Error);
kind!(FaultKind::Format, format; serde_json::Error, toml::de::Error, toml::ser::Error,
    std::string::FromUtf8Error, std::str::Utf8Error, std::num::ParseIntError,
    std::num::ParseFloatError, time::error::Format);
kind!(FaultKind::Bound, bound; std::num::TryFromIntError, time::error::ComponentRange);
kind!(FaultKind::Input, input; crate::Refusal);
kind!(FaultKind::Mismatch, mismatch;);
kind!(FaultKind::Credential, credential;);
kind!(FaultKind::Stream, stream; crate::agent::codex::ParseError);
kind!(FaultKind::Setup(SetupStep::Configuration), configuration;);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_the_detail_alone() {
        let fault = Fault::new(FaultKind::Setup(SetupStep::Seed), "seed refused: exact detail\n");
        assert_eq!(fault.to_string(), fault.detail);
    }

    #[test]
    fn a_missing_file_is_io() {
        let scratch = skein_scratch::Scratch::new("bench-fault");
        let fault = crate::read_result(&scratch.path().join("missing")).expect_err("missing file");
        assert_eq!(fault.kind, FaultKind::Io);
    }

    fn guard(root: &std::path::Path) -> crate::guard::Guard {
        crate::guard::Guard::new(&root.join("home"), &root.join("config"), &root.join("state"), &[]).expect("guard")
    }

    #[test]
    fn a_malformed_result_is_format() {
        let scratch = skein_scratch::Scratch::new("bench-fault-format");
        let path = scratch.path().join("result.json");
        std::fs::write(&path, "{malformed").expect("malformed result");
        assert_eq!(crate::read_result(&path).expect_err("result refused").kind, FaultKind::Format);
    }

    #[test]
    fn a_copied_arm_digest_mismatch_is_mismatch() {
        let scratch = skein_scratch::Scratch::new("bench-fault-arm");
        let root = scratch.path();
        let source = root.join("source");
        std::fs::write(&source, "unexecuted binary").expect("source bytes");
        let arms = root.join("arms");
        std::fs::create_dir(&arms).expect("arms");
        let arm = crate::arm::BuiltArm {
            version: 1,
            commit: "a".repeat(40),
            sha256: "0".repeat(64),
            toolchain: "test".into(),
            binaries: vec![crate::arm::ArmBinary { name: "smith".into(), sha256: "0".repeat(64) }],
        };
        let fault = crate::arm::archive(&guard(root), &arms, &[source], &arm).expect_err("copied bytes differ");
        assert_eq!(fault.kind, FaultKind::Mismatch);
    }

    #[test]
    fn a_refused_seed_names_the_seed_step() {
        let scratch = skein_scratch::Scratch::new("bench-fault-seed");
        let root = scratch.path();
        let seed = root.join("seed");
        std::fs::create_dir(&seed).expect("seed");
        std::fs::write(seed.join("file"), "frozen").expect("seed bytes");
        let fault = crate::isolate::prepare(&guard(root), &seed, &"0".repeat(64), &root.join("attempt"))
            .expect_err("wrong seed digest");
        assert_eq!(fault.kind, FaultKind::Setup(SetupStep::Seed));
    }

    #[test]
    fn a_refused_attempt_keeps_the_seed_step() {
        let scratch = skein_scratch::Scratch::new("bench-fault-attempt");
        let root = scratch.path();
        std::fs::create_dir(root.join("seed")).expect("seed");
        std::fs::write(root.join("seed/file"), "frozen").expect("seed bytes");
        let task = root.join("task.toml");
        std::fs::write(&task, format!("id='refused-seed'\nversion=1\nkind='probe'\ntier='small'\ndeadline_seconds=1\nprompt=\"A frozen task.\\n\"\ndeciding=['tree-ended']\nseed_sha256='{}'\n", "0".repeat(64)))
            .expect("task with wrong seed digest");
        let template = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/results/attempt.json");
        let mut identity = crate::read_result(&template).expect("template").identity;
        identity.run = "refused-seed".into();
        identity.attempt = "one".into();
        let prepared = crate::fixture::request::request(task, identity);
        let machine = crate::isolate::Machine { path: "/usr/bin:/bin".into(), rustup_home: None };
        let recorded =
            crate::run::attempt(&guard(root), &machine, &root.join("state"), &prepared).expect("record refusal");
        assert_eq!(recorded.incomplete, crate::run::Incomplete::Setup(SetupStep::Seed));
        let crate::End::Refused { setup } = recorded.result.outcome.end else { panic!("setup refusal") };
        assert!(setup.contains("seed digest differs"), "{setup}");
    }

    #[test]
    fn preparation_keeps_a_specific_step_and_exact_detail() {
        let fault = Fault::new(FaultKind::Setup(SetupStep::Credentials), "exact detail");
        assert_eq!(Fault::at(SetupStep::Configuration)(fault.clone()), fault);
        let fault = Fault::at(SetupStep::Warm)(Fault::new(FaultKind::Io, "exact detail"));
        assert_eq!(fault, Fault::new(FaultKind::Setup(SetupStep::Warm), "exact detail"));
    }
}
