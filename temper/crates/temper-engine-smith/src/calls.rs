//! Engine host-tool declarations and whole-object decoding. A malformed call
//! is feedback to Smith's LLM, never a root decision (domain/agent/tools.md).

use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, List, Overflow, Wall, Writer, bytes};
use smith_domain_run as run;
use temper_engine_domain as engine;

use crate::{Decoded, Problem, docs, forge, json::Value, nested, notes};

/// A host tool's description, at most (protocol/documentation.md, 8).
#[cfg(test)]
pub(crate) const DESCRIPTION_BYTES: usize = 160;
/// A member's description in a tool's schema, at most, at any depth.
#[cfg(test)]
pub(crate) const MEMBER_BYTES: usize = 120;

pub(crate) struct Spec {
    pub(crate) name: &'static [u8],
    pub(crate) description: &'static [u8],
    pub(crate) whole: Option<&'static [u8]>,
    pub(crate) schema: &'static [u8],
    pub(crate) effect: run::HostEffect,
    pub(crate) offer: Offer,
}

impl Spec {
    pub(crate) fn whole(&self) -> &'static [u8] {
        match self.whole {
            Some(schema) => schema,
            None => self.schema,
        }
    }
}

const FAMILIES: jig_domain_core::ToolFamilies = jig_domain_core::ToolFamilies::JIG;

/// One offering rule for declarations and their reference (protocol/documentation.md, 8).
#[derive(Clone, Copy)]
pub(crate) enum Offer {
    Every,
    Family(jig_domain_authority::Tools),
    Settings { asking: bool },
    Asking,
}

impl Offer {
    pub(crate) fn allows(self, charter: tasks::Tools, authority: tasks::Tools) -> bool {
        let families = jig_domain_core::ToolFamilies::JIG;
        let asking = engine::asking(charter);
        match self {
            Self::Every => !asking,
            Self::Family(family) => !asking && charter.0 & authority.0 & family.0 != 0,
            Self::Settings { asking: offered } => {
                (!asking || offered) && charter.0 & authority.0 & families.settings != 0
            }
            Self::Asking => asking && authority.0 & families.ask.0 != 0,
        }
    }

    pub(crate) fn words(self, out: &mut Writer) -> Result<(), Overflow> {
        let family = match self {
            Self::Every => return out.put(b"Offered to every run but an asking one."),
            Self::Asking => return out.put(b"Offered to an asking run, where its authority holds the asking family."),
            Self::Settings { .. } => b"settings".as_slice(),
            Self::Family(family) => family_name(family),
        };
        out.put(b"Offered where the run's charter names the ")?;
        out.put(family)?;
        out.put(b" family and its authority holds it.")
    }
}

fn family_name(family: jig_domain_authority::Tools) -> &'static [u8] {
    if family == FAMILIES.delegate {
        b"delegate"
    } else if family == FAMILIES.message {
        b"message"
    } else if family == FAMILIES.control {
        b"control"
    } else if family == FAMILIES.decide {
        b"decide"
    } else if family == FAMILIES.propose {
        b"propose"
    } else if family == FAMILIES.subscribe {
        b"subscribe"
    } else if family == FAMILIES.effect {
        b"effect"
    } else if family == FAMILIES.note {
        b"note"
    } else if family == FAMILIES.recall {
        b"recall"
    } else if family == FAMILIES.read {
        b"read"
    } else {
        unreachable!("offer holds a JIG family from delegate through read")
    }
}

const READ_FORGE_SPEC: Spec = Spec {
    name: b"read_forge",
    description: b"Repository reads; forge.client.rows/answer_bytes bounded. Compare names only; file by path. Each what: read_docs host-tools.",
    schema: br#"{"type":"object","properties":{"what":{"enum":["branch","branches","compare","file","protection","settings","collaborators","permission"],"description":"Needs: branches prefix; compare before, after; file path; permission user. No branch: the default branch."},"repository":{"type":"string"},"branch":{"type":"string"},"prefix":{"type":"string"},"before":{"type":"string"},"after":{"type":"string"},"commit":{"type":"string"},"path":{"type":"string"},"max_bytes":{"type":"integer"},"page":{"type":"integer"},"user":{"type":"integer"}},"required":["what"]}"#,
    whole: Some(br#"{"type":"object","properties":{"what":{"type":"string","enum":["branch","branches","compare","file","protection","settings","collaborators","permission"],"description":"Read to make; each what below lists its required and optional members."},"repository":{"type":"string","description":"owner/name; omit or empty for the project's one."},"branch":{"type":"string","description":"Branch name; omit or empty for the default branch. file: read at its head unless commit."},"prefix":{"type":"string","description":"Nonempty name prefix, as temper/; longer reads fewer."},"before":{"type":"string","description":"Earlier commit: 40 or 64 lowercase hex digits."},"after":{"type":"string","description":"Later commit: 40 or 64 lowercase hex digits."},"commit":{"type":"string","description":"Commit id: 40 or 64 lowercase hex digits. file: omit to read at branch's head."},"path":{"type":"string","description":"File or directory path; dot or empty for root."},"max_bytes":{"type":"integer","minimum":1,"description":"Raw prefix or log bytes; default: read limit. Leave room for metadata and escaping (up to 4x); cuts are incomplete.","maximum":4294967295},"page":{"type":"integer","minimum":1,"description":"Page from 1; default 1. Next page uncut; same page and skip if cut.","maximum":4294967295,"default":1},"user":{"type":"integer","minimum":1,"description":"Forge user's id; permission names their login.","maximum":18446744073709551615}},"required":["what"]}"#),
    effect: run::HostEffect::Read,
    offer: Offer::Family(FAMILIES.read),
};
const READ_ISSUES_SPEC: Spec = Spec {
    name: b"read_issues",
    description: b"Issues and pulls; forge.client.rows/answer_bytes bounded. Advance page; if cut, same page/skip. Each what: read_docs host-tools.",
    schema: br#"{"type":"object","properties":{"what":{"enum":["items","item","pull","pull_for","reviews","remarks","pull_files"],"description":"Needs: item, pull, reviews number; pull_for branch, base; remarks number, review; pull_files number, commit."},"repository":{"type":"string"},"since":{"type":"integer"},"page":{"type":"integer"},"item_kind":{"type":"string","enum":["issue","pull"]},"number":{"type":"integer"},"comment":{"type":"integer"},"branch":{"type":"string"},"base":{"type":"string"},"skip":{"type":"integer"},"review":{"type":"integer"},"commit":{"type":"string"}},"required":["what"]}"#,
    whole: Some(br#"{"type":"object","properties":{"what":{"type":"string","enum":["items","item","pull","pull_for","reviews","remarks","pull_files"],"description":"Read to make; each what below lists its required and optional members."},"repository":{"type":"string","description":"owner/name; omit or empty for the project's one."},"since":{"type":"integer","minimum":0,"description":"Items updated at or after this time in nanoseconds; default 0.","maximum":18446744073709551615,"default":0},"page":{"type":"integer","minimum":1,"description":"Page from 1; default 1. Next page uncut; same page and skip if cut.","maximum":4294967295,"default":1},"item_kind":{"type":"string","enum":["issue","pull"],"description":"issue or pull alone; both when absent."},"number":{"type":"integer","minimum":1,"description":"Issue or pull number.","maximum":18446744073709551615},"comment":{"type":"integer","minimum":0,"description":"Comments after this id; default 0.","maximum":18446744073709551615,"default":0},"branch":{"type":"string","description":"Branch name; pull_for: the branch it merges from."},"base":{"type":"string","description":"Branch it merges into."},"skip":{"type":"integer","minimum":0,"description":"Rows of page already read; default 0.","maximum":4294967295,"default":0},"review":{"type":"integer","minimum":1,"description":"Review id from read_issues what=reviews.","maximum":18446744073709551615},"commit":{"type":"string","description":"Commit id: 40 or 64 lowercase hex digits."}},"required":["what"]}"#),
    effect: run::HostEffect::Read,
    offer: Offer::Family(FAMILIES.read),
};
const READ_CI_SPEC: Spec = Spec {
    name: b"read_ci",
    description: b"Checks, statuses and job logs; forge.client.rows/answer_bytes bounded. Statuses by page; logs by max_bytes. Each what: read_docs host-tools.",
    schema: br#"{"type":"object","properties":{"what":{"enum":["checks","job","statuses"],"description":"Needs: checks, statuses commit; job commit, run, job, attempt."},"repository":{"type":"string"},"commit":{"type":"string"},"run":{"type":"integer"},"job":{"type":"integer"},"attempt":{"type":"integer"},"max_bytes":{"type":"integer"},"page":{"type":"integer"}},"required":["what"]}"#,
    whole: Some(br#"{"type":"object","properties":{"what":{"type":"string","enum":["checks","job","statuses"],"description":"Read to make; each what below lists its required and optional members."},"repository":{"type":"string","description":"owner/name; omit or empty for the project's one."},"commit":{"type":"string","description":"Commit id: 40 or 64 lowercase hex digits."},"run":{"type":"integer","minimum":1,"description":"Run id from read_ci what=checks or what=statuses; never a link's run number.","maximum":18446744073709551615},"job":{"type":"integer","minimum":0,"description":"Job id from read_ci what=checks or what=statuses; never a link's job index.","maximum":18446744073709551615},"attempt":{"type":"integer","minimum":0,"description":"Attempt from read_ci what=checks or what=statuses; may be 0 on Forgejo.","maximum":4294967295},"max_bytes":{"type":"integer","minimum":1,"description":"Raw prefix or log bytes; default: read limit. Leave room for metadata and escaping (up to 4x); cuts are incomplete.","maximum":4294967295},"page":{"type":"integer","minimum":1,"description":"Page from 1; default 1. Next page uncut; same page and skip if cut.","maximum":4294967295,"default":1}},"required":["what"]}"#),
    effect: run::HostEffect::Read,
    offer: Offer::Family(FAMILIES.read),
};

const SEE: Spec = Spec {
    name: b"see",
    description: b"Read temper by address, as your person, in your project: its work, changes, settings, proposals, pages and files. Each answer ends with what to read next.",
    schema: br#"{"type":"object","additionalProperties":false,"required":["address"],"properties":{"address":{"type":"string","description":"One line, as your instructions list addresses: settings charter 1, help, more TOKEN."}}}"#,
    effect: run::HostEffect::Read, whole: None, offer: Offer::Asking,
};

const DOCUMENT: Spec = Spec {
    name: b"document",
    description: b"Keep this chat's one document: write it whole first, its first line # TITLE; then append a section, or replace the one under the same ## heading.",
    schema: br#"{"type":"object","additionalProperties":false,"required":["action","body"],"properties":{"action":{"enum":["write","append","replace"],"description":"write: the whole document; append: a section at its end; replace: the section under body's first ## heading."},"body":{"type":"string","description":"Markdown: the whole document, or one section beginning with its ## heading."}}}"#,
    effect: run::HostEffect::Write, whole: None, offer: Offer::Asking,
};

const REMEMBER: Spec = Spec {
    name: b"remember",
    description: b"Keep a line about your person across their chats, only when they ask: add one, or change or drop one by its number. Each answer is the whole list.",
    schema: br#"{"type":"object","additionalProperties":false,"required":["action"],"properties":{"action":{"enum":["add","change","drop"],"description":"The memory act."},"line":{"type":"integer","minimum":1,"maximum":4294967295,"description":"The line's number, for change and drop."},"text":{"type":"string","description":"One line, at most 120 bytes."}}}"#,
    effect: run::HostEffect::Write, whole: None, offer: Offer::Asking,
};

const ASK_PROPOSE_SETTINGS_WHOLE: &[u8] = br#"{"type":"object","additionalProperties":false,"properties":{"against":{"type":"integer","minimum":1,"maximum":4294967295,"description":"The newest revision you read. A proposal against an older one is stale, and you hear the newest."},"reason":{"type":"string","description":"Why; the owners read it."},"items":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"setting":{"type":"object","properties":{"kind":{"enum":["charter","template","policy","rules","model","worker","provider","landing-rule","forge-charter","template-goal","project-policy","project-ways","project-questions","project-classes","charter-words","project-words","follow","dependency","project-areas","project-triager","welcome","agents"],"description":"The setting's kind."},"number":{"type":"integer","minimum":1,"description":"Setting number; template-goal: its template's. Omit for rules, welcome and unnumbered project settings."},"project":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Project settings: required; landing-rule: optional, absent=deployment; charter-words/template-goal/welcome: none."}},"required":["kind"],"description":"The setting, named as a settings file names it."},"set":{"description":"Its whole value as a settings file holds it; forge-charter takes a number; a document's repository is OWNER/NAME."},"edit":{"enum":["reset","removed"],"description":"In place of set: reset, to follow its default again, or removed, no longer offered."},"remove":{"type":"boolean","description":"true instead of set or edit: withdraw the setting."},"reset":{"type":"boolean","description":"true instead of set or edit: follow its default."}},"required":["setting"]},"description":"The changes, at most one per setting, in this call's part."},"more":{"type":"boolean","description":"true: hold these items for the next call; the call without it proposes them all."}},"required":["against","reason","items"]}"#;

const PLAN_CHANGE_WHOLE: &[u8] = br#"{"type":"object","properties":{"words":{"type":"string","description":"Change's complete brief.","minLength":1},"repository":{"type":"string","description":"owner/name; omit if sole writable."},"name":{"type":"string","description":"A name for after."},"landing":{"type":"string","description":"goal/name; omit: held goal/base."},"branch":{"type":"string","enum":["mine"],"description":"mine: goal into base; omit: a new."},"merge":{"type":"string","description":"Catch-up: 40/64 hex commit."},"after":{"type":"array","description":"Live delegate IDs/earlier names.","items":{"oneOf":[{"type":"integer","minimum":1},{"type":"string"}]}},"producer":{"type":"integer","minimum":1,"description":"Producer charter; omit: forge default."},"llm":{"type":"object","description":"producer/review:{main:{model,effort},sub_agents:[{model,effort}]}."},"changes":{"type":"array","description":"Batch: put all options inside each item.","items":{"type":"object","properties":{"words":{"type":"string","description":"Change's complete brief.","minLength":1},"repository":{"type":"string","description":"owner/name; omit if sole writable."},"name":{"type":"string","description":"A name for after."},"landing":{"type":"string","description":"goal/name; omit: held goal/base."},"branch":{"type":"string","enum":["mine"],"description":"mine: goal into base; omit: a new."},"merge":{"type":"string","description":"Catch-up: 40/64 hex commit."},"after":{"type":"array","description":"Live delegate IDs/earlier names.","items":{"oneOf":[{"type":"integer","minimum":1},{"type":"string"}]}},"producer":{"type":"integer","minimum":1,"description":"Producer charter; omit: forge default."},"llm":{"type":"object","description":"producer/review:{main:{model,effort},sub_agents:[{model,effort}]}."},"repairs":{"type":"string","description":"Red landing's 40/64 hex merge commit.","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"},"questions":{"type":"string","enum":["none","up"],"description":"up, none; omit: project setting."}},"required":["words"]},"minItems":1},"repairs":{"type":"string","description":"Red landing's 40/64 hex merge commit.","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"},"questions":{"type":"string","enum":["none","up"],"description":"up, none; omit: project setting."}}}"#;

const DELEGATE_WHOLE: &[u8] = br#"{"type":"object","properties":{"batch":{"type":"array","description":"The tasks: each with executor, spec, contract, authority; optionally deadline_seconds, tracked, dependencies, wake, llm.","items":{"type":"object","properties":{"executor":{"type":"object","description":"{kind: agent, charter}, {kind: procedure, connector, code}, {kind: person, person} or {kind: role, role}.","properties":{"kind":{"type":"string","enum":["agent","procedure","person","role"]},"charter":{"type":"integer","minimum":0},"connector":{"type":"integer","minimum":0},"code":{"type":"integer","minimum":0},"person":{"type":"integer","minimum":0},"role":{"type":"integer","minimum":0}},"required":["kind"]},"spec":{"type":"object","properties":{"words":{"type":"string"},"parameters":{"type":"array","items":{"type":"object","description":"{name, kind: number, value}, {name, kind: bytes, value} or {name, kind: resource, connector, resource}.","properties":{"name":{"type":"integer","minimum":0},"kind":{"type":"string","enum":["number","bytes","resource"]},"value":{},"connector":{"type":"integer","minimum":0},"resource":{"type":"integer","minimum":0}},"required":["name","kind"]}},"inputs":{"type":"array","items":{"type":"integer","minimum":0}}},"required":["words"]},"contract":{"type":"object","description":"Report: words; change: connector,change_kind,words; verdict: choices. A person's or a role's report: words at least 1.","properties":{"kind":{"type":"string","enum":["report","change","verdict"]},"words":{"type":"integer","minimum":0,"description":"A person's or a role's report: words at least 1."},"connector":{"type":"integer","minimum":0},"change_kind":{"type":"integer","minimum":0},"choices":{"type":"array","items":{"type":"object","properties":{"code":{"type":"integer","minimum":0},"words":{"type":"integer","minimum":0}},"required":["code","words"]}}},"required":["kind"]},"authority":{"type":"object","description":"Whole: restate every member you keep.","properties":{"tools":{"type":"integer","description":"The tool families, as a bit set.","minimum":0},"grants":{"type":"array","items":{"type":"object","properties":{"connector":{"type":"integer","minimum":0},"kind":{"type":"integer","minimum":0},"segments":{"type":"array","items":{"type":"string"}},"terminal":{"type":"string","enum":["exact","open"]},"last":{"type":"string"}},"required":["connector","kind","segments","terminal","last"]}},"delegation":{"type":"object","properties":{"kinds":{"type":"array","items":{"type":"object","properties":{"kind":{"type":"string","enum":["agent","procedure","role"]},"number":{"type":"integer","minimum":0}},"required":["kind","number"]}},"tasks":{"type":"integer","minimum":0},"depth":{"type":"integer","minimum":0}},"required":["kinds","tasks","depth"]},"budget":{"type":"object","properties":{"spend":{"type":"integer","minimum":0},"deadline":{"type":"integer","description":"Wall-clock nanoseconds since the Unix epoch.","minimum":0}},"required":["spend"]},"notes":{"type":"integer","minimum":0,"maximum":255},"note_resources":{"type":"array","items":{"type":"object","properties":{"connector":{"type":"integer","minimum":0},"pattern":{"type":"object","properties":{"segments":{"type":"array","items":{"type":"string"}},"terminal":{"type":"string","enum":["exact","open"]},"last":{"type":"string"}},"required":["segments","terminal","last"]}},"required":["connector","pattern"]}}},"required":["tools","grants","delegation","budget","notes"]},"deadline_seconds":{"type":"integer","description":"It must end this many seconds after it is made. Give this or authority.budget.deadline; with neither, it ends with you.","minimum":1},"tracked":{"type":"object","description":"Track it as a goal at this priority, the lower first. Leave it out for an untracked task.","properties":{"priority":{"type":"integer","minimum":0,"maximum":4294967295}},"required":["priority"]},"dependencies":{"type":"array","items":{"type":"object","properties":{"kind":{"type":"string","enum":["batch","existing"]},"number":{"type":"integer","minimum":0}},"required":["kind","number"]}},"wake":{"type":"object","properties":{"words":{"type":"object","properties":{"kind":{"type":"string","enum":["never","immediate","batch"]},"count":{"type":"integer","minimum":0},"age":{"type":"integer","description":"Nanoseconds.","minimum":0}},"required":["kind"]},"notices":{"type":"object","properties":{"kind":{"type":"string","enum":["never","immediate","batch"]},"count":{"type":"integer","minimum":0},"age":{"type":"integer","description":"Nanoseconds.","minimum":0}},"required":["kind"]},"news":{"type":"object","properties":{"kind":{"type":"string","enum":["never","immediate","batch"]},"count":{"type":"integer","minimum":0},"age":{"type":"integer","description":"Nanoseconds.","minimum":0}},"required":["kind"]},"results":{"type":"string","enum":["never","each","last_or_failure"]},"questions":{"type":"boolean"},"answers":{"type":"boolean"},"timers":{"type":"boolean"}},"required":["words","notices","news","results","questions","answers","timers"]},"llm":{"type":"object","description":"main {model, effort} names the model, effort optional; subagents and account are optional.","properties":{"main":{"type":"object","properties":{"model":{"type":"integer","minimum":0,"maximum":255},"effort":{"type":"integer","minimum":0,"maximum":255}},"required":["model"]},"subagents":{"type":"array","items":{"type":"object","properties":{"model":{"type":"integer","minimum":0,"maximum":255},"effort":{"type":"integer","minimum":0,"maximum":255}},"required":["model"]}},"account":{"type":"integer","minimum":0}},"required":["main"]},"symbolic_grants":{"type":"array","items":{"type":"object","properties":{"connector":{"type":"integer","minimum":0},"kind":{"type":"integer","minimum":0},"segments":{"type":"array","items":{"type":"string"}},"terminal":{"type":"string","enum":["exact","open"]},"last":{"type":"string"}},"required":["connector","kind","segments","terminal","last"]}}},"required":["executor","spec","contract","authority"]}}},"required":["batch"]}"#;

const AMEND_WHOLE: &[u8] = br#"{"type":"object","examples":[{"target":3,"revision":1,"amendment":{"reason":"Finish by Friday.","authority":{"tools":5,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":1000,"deadline":1793404800000000000},"notes":0}}}],"properties":{"target":{"type":"integer","description":"The delegate's number.","minimum":1},"revision":{"type":"integer","description":"The spec revision you last read. Leave it out only to change a tracked delegate's priority alone.","minimum":0},"amendment":{"type":"object","description":"The reason, which its live run hears, and what changes: tracked, instructions, spec, authority, dependencies, wake, llm.","properties":{"reason":{"type":"string","description":"What the delegate's live run hears, unless new instructions replace it."},"tracked":{"description":"{priority} tracks it as a goal, the lower first; untracked stops tracking it.","oneOf":[{"type":"object","properties":{"priority":{"type":"integer","minimum":0,"maximum":4294967295}},"required":["priority"]},{"type":"string","enum":["untracked"]}]},"instructions":{"type":"string","description":"Its own instructions, whole; empty leaves it none."},"spec":{"type":"object","properties":{"words":{"type":"string"},"parameters":{"type":"array","items":{"type":"object","description":"{name, kind: number, value}, {name, kind: bytes, value} or {name, kind: resource, connector, resource}.","properties":{"name":{"type":"integer","minimum":0},"kind":{"type":"string","enum":["number","bytes","resource"]},"value":{},"connector":{"type":"integer","minimum":0},"resource":{"type":"integer","minimum":0}},"required":["name","kind"]}},"inputs":{"type":"array","items":{"type":"integer","minimum":0}}},"required":["words"]},"authority":{"type":"object","description":"Whole: restate every member you keep. A deadline is budget.deadline.","properties":{"tools":{"type":"integer","description":"The tool families, as a bit set.","minimum":0},"grants":{"type":"array","items":{"type":"object","properties":{"connector":{"type":"integer","minimum":0},"kind":{"type":"integer","minimum":0},"segments":{"type":"array","items":{"type":"string"}},"terminal":{"type":"string","enum":["exact","open"]},"last":{"type":"string"}},"required":["connector","kind","segments","terminal","last"]}},"delegation":{"type":"object","properties":{"kinds":{"type":"array","items":{"type":"object","properties":{"kind":{"type":"string","enum":["agent","procedure","role"]},"number":{"type":"integer","minimum":0}},"required":["kind","number"]}},"tasks":{"type":"integer","minimum":0},"depth":{"type":"integer","minimum":0}},"required":["kinds","tasks","depth"]},"budget":{"type":"object","properties":{"spend":{"type":"integer","minimum":0},"deadline":{"type":"integer","description":"Wall-clock nanoseconds since the Unix epoch.","minimum":0}},"required":["spend"]},"notes":{"type":"integer","minimum":0,"maximum":255},"note_resources":{"type":"array","items":{"type":"object","properties":{"connector":{"type":"integer","minimum":0},"pattern":{"type":"object","properties":{"segments":{"type":"array","items":{"type":"string"}},"terminal":{"type":"string","enum":["exact","open"]},"last":{"type":"string"}},"required":["segments","terminal","last"]}},"required":["connector","pattern"]}}},"required":["tools","grants","delegation","budget","notes"]},"dependencies":{"type":"array","description":"The whole list of the existing tasks it waits on.","items":{"type":"integer","minimum":1}},"wake":{"type":"object","properties":{"words":{"type":"object","properties":{"kind":{"type":"string","enum":["never","immediate","batch"]},"count":{"type":"integer","minimum":0},"age":{"type":"integer","description":"Nanoseconds.","minimum":0}},"required":["kind"]},"notices":{"type":"object","properties":{"kind":{"type":"string","enum":["never","immediate","batch"]},"count":{"type":"integer","minimum":0},"age":{"type":"integer","description":"Nanoseconds.","minimum":0}},"required":["kind"]},"news":{"type":"object","properties":{"kind":{"type":"string","enum":["never","immediate","batch"]},"count":{"type":"integer","minimum":0},"age":{"type":"integer","description":"Nanoseconds.","minimum":0}},"required":["kind"]},"results":{"type":"string","enum":["never","each","last_or_failure"]},"questions":{"type":"boolean"},"answers":{"type":"boolean"},"timers":{"type":"boolean"}},"required":["words","notices","news","results","questions","answers","timers"]},"llm":{"type":"object","description":"main {model, effort} names the model, effort optional; subagents and account are optional.","properties":{"main":{"type":"object","properties":{"model":{"type":"integer","minimum":0,"maximum":255},"effort":{"type":"integer","minimum":0,"maximum":255}},"required":["model"]},"subagents":{"type":"array","items":{"type":"object","properties":{"model":{"type":"integer","minimum":0,"maximum":255},"effort":{"type":"integer","minimum":0,"maximum":255}},"required":["model"]}},"account":{"type":"integer","minimum":0}},"required":["main"]}},"required":["reason"]}},"required":["target","amendment"]}"#;

const SETTINGS_WHOLE: &[u8] = br#"{"type":"object","additionalProperties":false,"properties":{"setting":{"type":"object","additionalProperties":false,"properties":{"kind":{"enum":["charter","template","policy","rules","model","worker","provider","landing-rule","forge-charter","template-goal","project-policy","project-ways","project-questions","project-classes","charter-words","project-words","follow","dependency","project-areas","project-triager","welcome","agents"],"description":"The setting's kind."},"number":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Setting number; template-goal: its template's. Omit for rules, welcome and unnumbered project settings."},"project":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Project settings: required; landing-rule: optional, absent=deployment; charter-words/template-goal/welcome: none."}},"required":["kind"],"description":"One setting's whole value, named as a settings file names it."},"revision":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Alone, that revision's record: its items and who decided it. With a charter, the charter as that held revision has it."},"history":{"type":"object","additionalProperties":false,"properties":{"before":{"type":"integer","minimum":1,"maximum":4294967295,"description":"The page before this revision; the newest page when absent."}},"description":"A page of the revisions, the newest first."},"from":{"type":"integer","minimum":0,"maximum":4294967295,"description":"Where to go on in the answer's text: the offset after next: on its last line; 0 when absent."}}}"#;

const PROPOSE_SETTINGS_WHOLE: &[u8] = br#"{"type":"object","additionalProperties":false,"properties":{"against":{"type":"integer","minimum":1,"maximum":4294967295,"description":"The newest revision you read. A proposal against an older one is stale, and you hear the newest."},"reason":{"type":"string","description":"Why; the owners read it."},"note":{"type":"object","additionalProperties":false,"properties":{"name":{"type":"integer","minimum":1},"revision":{"type":"integer","minimum":1,"maximum":4294967295}},"required":["name","revision"],"description":"{name, revision}: goal note work: settings with {items:[...]}; build a few KB per write with first/remove/join."},"items":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"setting":{"type":"object","properties":{"kind":{"enum":["charter","template","policy","rules","model","worker","provider","landing-rule","forge-charter","template-goal","project-policy","project-ways","project-questions","project-classes","charter-words","project-words","follow","dependency","project-areas","project-triager","welcome","agents"],"description":"The setting's kind."},"number":{"type":"integer","minimum":1,"description":"Setting number; template-goal: its template's. Omit for rules, welcome and unnumbered project settings."},"project":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Project settings: required; landing-rule: optional, absent=deployment; charter-words/template-goal/welcome: none."}},"required":["kind"],"description":"The setting, named as a settings file names it."},"set":{"description":"Its whole new value, as a settings file holds it; forge-charter takes a charter number. Read the setting first."},"edit":{"enum":["reset","removed"],"description":"In place of set: reset, to follow its default again, or removed, no longer offered."},"remove":{"type":"boolean","description":"true instead of set or edit: withdraw the setting."},"reset":{"type":"boolean","description":"true instead of set or edit: follow its default."}},"required":["setting"]},"description":"The changes, at most one per setting. Supply exactly one of items or note."}},"required":["against","reason"]}"#;

const ASK_PROPOSE_SETTINGS: Spec = Spec {
    name: b"propose_settings",
    description: b"Propose whole settings against newest revision read; an owner decides before anything changes. Large sets take parts.",
    schema: br#"{"type":"object","additionalProperties":false,"properties":{"against":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Newest revision read; older is stale."},"reason":{"type":"string","description":"Why."},"items":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"setting":{"type":"object","description":"Identity (agents/worker/provider). Full: read_docs host-tools, propose_settings setting."},"set":{"description":"Whole settings-file value; forge-charter: number."},"edit":{"enum":["reset","removed"],"description":"Instead of set: reset/default or remove."},"remove":{"type":"boolean"},"reset":{"type":"boolean"}},"required":["setting"]},"description":"Changes; at most one per setting."},"more":{"type":"boolean","description":"Hold for next call; absent: propose all parts."}},"required":["against","reason","items"]}"#,
    effect: run::HostEffect::Write, whole: Some(ASK_PROPOSE_SETTINGS_WHOLE), offer: Offer::Settings { asking: true },

};

const PLAN_CHANGE: Spec = Spec { name: b"change", description: b"Have a project repository change written on a branch, opened as a pull request, checked and merged; hear its result.", schema: br#"{"type":"object","properties":{"words":{"type":"string","description":"Brief.","minLength":1},"repository":{"type":"string","description":"owner/name; omit: sole writable."},"producer":{"type":"integer","minimum":1,"description":"Charter; omit: default."},"questions":{"type":"string","enum":["none","up"],"description":"Omit: project default."},"changes":{"type":"array","minItems":1,"items":{"type":"object","properties":{"words":{"type":"string","description":"Brief.","minLength":1},"producer":{"type":"integer","minimum":1,"description":"Charter; omit: default."},"questions":{"type":"string","enum":["none","up"],"description":"Omit: project default."}},"required":["words"]},"description":"Batch; options per item. More: read_docs search \"change changes\" on host-tools."}},"description":"More: name, landing, after, branch, merge, llm, repairs. read_docs search \"change\" on host-tools."}"#, effect: run::HostEffect::Write, whole: Some(PLAN_CHANGE_WHOLE), offer: Offer::Family(FAMILIES.delegate) };

pub(crate) const SPECS: [Spec; 32] = [
    crate::finding::SPEC,
    crate::red::SPEC,
    crate::follow::SPEC,
    Spec { name: b"decided", description: b"Ask whether you may decide for your person. Carry on only if decided; otherwise the answer says how to ask.", schema: br#"{"type":"object","additionalProperties":false,"properties":{"question":{"type":"string","description":"Question, up to 400 bytes."},"options":{"type":"array","minItems":2,"maxItems":6,"items":{"type":"string"},"description":"2-6 options, each up to 120 bytes."},"took":{"type":"integer","minimum":0,"maximum":5,"description":"Chosen index, from 0."},"why":{"type":"string","description":"Why, up to 400 bytes."},"kind":{"type":"string","enum":["wording","retry","implementation","tests","dependency","departure","money","irreversible","rules","other","findings"],"description":"Kind of choice."},"paths":{"type":"array","maxItems":16,"items":{"type":"string"},"description":"Up to 16 repository-relative paths, each 1-256 bytes, no leading /."},"under":{"type":"integer","minimum":1,"maximum":20,"description":"Covering item in the project's classes."}},"required":["question","options","took","why","kind"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.control) },
    Spec { name: b"goal", description: b"Have one goal tracked, only for work your person wants agreed first (set agree) or a survey that changes nothing. For a change to a repository, call change.", schema: br#"{"type":"object","additionalProperties":false,"properties":{"words":{"type":"string","description":"The goal's complete brief."},"advisor":{"type":"boolean","description":"Set by a goal alone, for a survey that changes nothing."},"agree":{"type":"boolean","description":"Set by the person's chat when they want the work agreed first: the goal asks its plan before any change."}},"required":["words"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.delegate) },
    Spec { name: b"change", description: b"Have one change written on a branch of a repository of this project, opened as a pull request and merged once its checks pass; you hear its result.", schema: br#"{"type":"object","properties":{"words":{"type":"string","description":"What to change: the files, and the exact text where it matters. It is all the change's coder reads."},"repository":{"type":"string","description":"The repository, as owner/name; leave it out when the project has one you may write."}},"required":["words"]}"#, effect: run::HostEffect::Write, whole: Some(PLAN_CHANGE_WHOLE), offer: Offer::Family(FAMILIES.delegate) },
    Spec { name: b"delegate", description: b"Batch direct delegates within authority. tracked {priority}: goals, lower first. Repository changes: call change.", schema: br#"{"type":"object","properties":{"batch":{"type":"array","description":"Tasks.","items":{"type":"object","properties":{"executor":{"type":"object","description":"Whole form: read_docs search \"delegate executor\" on host-tools."},"spec":{"type":"object","properties":{"words":{"type":"string"}},"required":["words"],"description":"More: parameters, inputs. read_docs search \"delegate spec\" on host-tools."},"contract":{"type":"object","description":"Whole form: read_docs search \"delegate contract\" on host-tools."},"authority":{"type":"object","description":"Whole: tools, grants, delegation, budget, notes. Whole form: read_docs search \"delegate authority\" on host-tools."}},"required":["executor","spec","contract","authority"],"description":"More: deadline_seconds, tracked, dependencies, wake, llm, symbolic_grants. read_docs search \"delegate\" on host-tools."}}},"required":["batch"]}"#, effect: run::HostEffect::Write, whole: Some(DELEGATE_WHOLE), offer: Offer::Family(FAMILIES.delegate) },
    Spec { name: b"amend", description: b"Amend a live delegate at its spec revision within authority. Omitted parameters remain; lists replace. Authority and instructions are whole.", schema: br#"{"type":"object","properties":{"target":{"type":"integer","description":"Delegate number.","minimum":1},"revision":{"type":"integer","description":"Spec revision read; omit only for priority alone.","minimum":0},"amendment":{"type":"object","description":"More: tracked, spec, dependencies, wake, llm. read_docs search \"amend amendment\" on host-tools.","properties":{"reason":{"type":"string","description":"What its live run hears."},"instructions":{"type":"string","description":"Whole instructions; empty clears."},"authority":{"type":"object","description":"Whole replacement. Whole form: read_docs search \"amend authority\" on host-tools."}},"required":["reason"]}},"required":["target","amendment"]}"#, effect: run::HostEffect::Write, whole: Some(AMEND_WHOLE), offer: Offer::Family(FAMILIES.control) },
    Spec { name: b"message", description: b"Send words, a question or an answer to a referenced task; its run reads it next turn.", schema: br#"{"type":"object","properties":{"target":{"type":"integer","description":"Task.","minimum":1},"form":{"type":"string","description":"An answer names its question.","enum":["words","question","answer"]},"words":{"type":"string","description":"Text."},"question":{"type":"integer","description":"Answer: question ID.","minimum":1}},"required":["target","form","words"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.message) },
    Spec { name: b"cancel", description: b"Cancel a live delegate and every task below it, with a reason.", schema: br#"{"type":"object","properties":{"target":{"type":"integer","description":"The delegate's number.","minimum":1},"reason":{"type":"string","description":"Why it is cancelled."}},"required":["target","reason"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.control) },
    Spec { name: b"release", description: b"Release a held delegate, naming its hold revision. One a party's hold holds is theirs to release: propose it.", schema: br#"{"type":"object","properties":{"target":{"type":"integer","description":"The held delegate's number.","minimum":1},"revision":{"type":"integer","description":"The hold's revision, as you last read it.","minimum":0},"again":{"type":"boolean","description":"For an effect held uncertain alone: true makes it again, false does not."}},"required":["target","revision"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.control) },
    Spec { name: b"introduce", description: b"Give two referenced tasks references to each other, so each can message the other.", schema: br#"{"type":"object","properties":{"left":{"type":"integer","description":"Task number.","minimum":1},"right":{"type":"integer","description":"Other task.","minimum":1}},"required":["left","right"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.message) },
    Spec { name: b"subscribe", description: b"Hear of a referenced task held/ended, or a timer once/periodically. Returns subscription number.", schema: br#"{"type":"object","properties":{"kind":{"type":"string","description":"task or timer.","enum":["task","timer"]},"target":{"type":"integer","description":"task: task number.","minimum":1},"held":{"type":"boolean","description":"task: hear when held; omit: false."},"result":{"type":"boolean","description":"task: hear result; omit: true."},"at":{"type":"integer","description":"timer: Unix-epoch wall nanoseconds.","minimum":0},"period":{"type":"integer","description":"timer: repeat interval, nanoseconds; omit: once.","minimum":1}},"required":["kind"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.subscribe) },
    Spec { name: b"unsubscribe", description: b"Remove one of your subscriptions.", schema: br#"{"type":"object","properties":{"subscription":{"type":"integer","description":"The subscription's number, as subscribe answered it.","minimum":1}},"required":["subscription"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.subscribe) },
    Spec { name: b"decide", description: b"Decide a proposal waiting for you: accept it, reject it with a reason, or pass it up to the holder above.", schema: br#"{"type":"object","properties":{"proposer":{"type":"integer","description":"The proposing task's number.","minimum":1},"proposal":{"type":"integer","description":"The proposal's number among its proposer's.","minimum":1},"decision":{"type":"string","description":"accept, reject or pass.","enum":["accept","reject","pass"]},"reason":{"type":"string","description":"Why; required to reject."}},"required":["proposer","proposal","decision"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.decide) },
    Spec { name: b"decide_escalation", description: b"Decide a held descendant's escalation: release it, reject it with a reason, or pass it up to the holder above.", schema: br#"{"type":"object","properties":{"task":{"type":"integer","description":"The held task's number.","minimum":1},"revision":{"type":"integer","description":"Its hold's revision, as you last read it.","minimum":1},"decision":{"type":"string","description":"release, reject or pass.","enum":["release","reject","pass"]},"reason":{"type":"string","description":"Why; required to reject."}},"required":["task","revision","decision"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.decide) },
    Spec { name: b"propose", description: b"Ask holder beyond authority.", schema: br#"{"type":"object","properties":{"action":{"type":"string","description":"Act.","enum":["batch","effect","amend","widen","release"]},"task":{"type":"integer","description":"amend/widen/release: task.","minimum":1},"revision":{"type":"integer","description":"amend: spec; release: hold.","minimum":0},"hold":{"type":"integer","minimum":1,"description":"release: party's hold; await its owner."},"batch":{"type":"array","description":"delegate's batch.","items":{"type":"object"}},"amendment":{"type":"object","description":"amend's amendment."},"authority":{"type":"object","description":"Whole authority."},"reason":{"type":"string","description":"Why."},"as_holder":{"type":"boolean","description":"batch: accepter holds them."},"repository":{"type":"object","description":"effect: effect_forge.repository."},"resource":{"type":"object","description":"effect: effect_forge.resource."},"write":{"type":"object","description":"effect: effect_forge.write."}},"required":["action","reason"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.propose) },
    Spec { name: b"withdraw", description: b"Withdraw one of your pending proposals.", schema: br#"{"type":"object","properties":{"proposal":{"type":"integer","description":"The proposal's number, as propose answered it.","minimum":1}},"required":["proposal"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.decide) },
    Spec { name: b"note", description: b"Write or revise a note, whole or by lines, at the revision read.", schema: br#"{"type":"object","properties":{"scope":{"type":"object","description":"{kind}: deployment; project:+project; goal:+project,goal; resources:+project,connector,pattern."},"description":{"type":"string","description":"What a search reads."},"body":{"type":"string","description":"New text or range replacement."},"references":{"type":"array","description":"Task numbers.","items":{"type":"integer","minimum":1}},"name":{"type":"integer","description":"Revise: name; omit: new.","minimum":1},"recalled":{"type":"integer","description":"Revision read.","minimum":1},"first":{"type":"integer","description":"With name,recalled,remove: first line (1-based); remove=0 past end appends.","minimum":1},"remove":{"type":"integer","description":"With first: lines removed; 0 inserts.","minimum":0},"join":{"type":"boolean","description":"With first,remove: join preceding line without newline."}},"required":["scope","description","body"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.note) },
    Spec { name: b"recall", description: b"Recall a note by its name, or search the descriptions of the notes in some scopes, a page at a time.", schema: br#"{"type":"object","properties":{"kind":{"type":"string","description":"name, by its name; search, by words in scopes.","enum":["name","search"]},"name":{"type":"integer","description":"For name: the note's name.","minimum":1},"scopes":{"type":"array","description":"For search: the scopes, each as note's scope.","items":{"type":"object"}},"query":{"type":"string","description":"For search: the words."},"page":{"type":"integer","description":"The page, from 0; 0 when absent.","minimum":0}},"required":["kind"]}"#, effect: run::HostEffect::Read, whole: None, offer: Offer::Family(FAMILIES.recall) },
    Spec { name: b"subscribe_forge", description: b"Hear forge news: landings on a branch, CI on heads, pull request state, or comments and reviews on an issue or pull request.", schema: br#"{"type":"object","properties":{"topic":{"type":"object","description":"{kind,repository?}: owner/name|{forge,repository}, else sole readable; landings+branch,ci+head,pull/participation+number"},"own_change":{"type":"integer","description":"For landings: your change task's number; its own landing does not wake you.","minimum":1},"paths":{"type":"array","description":"For landings: a landing wakes you when it changes one of these paths, or a file below one.","items":{"type":"string"}}},"required":["topic"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.subscribe) },
    Spec { name: b"effect_forge", description: b"Make one write on an adopted repository through the forge's connector: an issue, a comment or a status, among the forge's writes.", schema: br#"{"type":"object","properties":{"repository":{"type":"object","description":"The repository, as {forge, repository}: the forge's number and the repository's."},"resource":{"type":"object","description":"What it writes on: {kind: repository}, {kind: pull, number}, {kind: issue, number} or {kind: branch, segments}."},"write":{"type":"object","description":"{kind: create_issue, key, title, body}, {kind: post, number, key, body} or {kind: status, commit, context, check}."}},"required":["repository","resource","write"]}"#, effect: run::HostEffect::Write, whole: None, offer: Offer::Family(FAMILIES.effect) },

    Spec {
        name: b"read_task",
        description: b"Specs/results; from=next. For an answer: durable question numbers and answerers.",
        schema: br#"{"type":"object","additionalProperties":false,"properties":{"task":{"type":"integer","minimum":1,"description":"Task number."},"field":{"enum":["specification","result"],"description":"Specification or result; omit for live state."},"from":{"type":"integer","minimum":0,"default":0,"description":"Byte offset from next; default 0."}},"required":["task"]}"#,
        effect: run::HostEffect::Read, whole: None, offer: Offer::Family(FAMILIES.read),
    },
    Spec {
        name: b"settings", description: b"Read settings: no members gives inventory; setting, revision or history selects one. Long answers continue from an offset.",
        schema: br#"{"type":"object","additionalProperties":false,"properties":{"setting":{"type":"object","description":"Setting identity, agents, worker and provider included. Whole form: read_docs search \"settings setting\" on host-tools."},"revision":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Alone: revision record; with charter: its held revision."},"history":{"type":"object","additionalProperties":false,"properties":{"before":{"type":"integer","minimum":1,"maximum":4294967295,"description":"The page before this revision; the newest page when absent."}},"description":"A page of the revisions, the newest first."},"from":{"type":"integer","minimum":0,"maximum":4294967295,"description":"Offset after next:; omit: 0."}}}"#,
        effect: run::HostEffect::Read, whole: Some(SETTINGS_WHOLE), offer: Offer::Settings { asking: false },
    },
    Spec {
        name: b"propose_settings", description: b"Propose whole settings from items/note revision; owner acceptance makes a revision. Keep your document separate.",
        schema: br#"{"type":"object","properties":{"against":{"type":"integer","minimum":1,"maximum":4294967295,"description":"Newest revision read; older is stale."},"reason":{"type":"string","description":"Why."},"items":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"setting":{"type":"object","description":"Identity (agents/worker/provider). Full: read_docs host-tools, propose_settings setting."},"set":{"description":"Whole settings-file value; forge-charter: number. Read it first."},"edit":{"enum":["reset","removed"],"description":"Instead of set: reset/default or remove."},"remove":{"type":"boolean"},"reset":{"type":"boolean"}},"required":["setting"]},"description":"Changes; at most one per setting. Exactly one of items or note."}},"required":["against","reason"],"description":"Also note. Whole form: read_docs search \"propose_settings note\" on host-tools."}"#,
        effect: run::HostEffect::Write, whole: Some(PROPOSE_SETTINGS_WHOLE), offer: Offer::Settings { asking: true },

    },
    READ_FORGE_SPEC,
    READ_ISSUES_SPEC,
    READ_CI_SPEC,
    Spec { name: b"read_docs", description: b"Read pages: {topic,from} gives text then next: OFFSET or end; {search} finds sections holding every word.", schema: br#"{"type":"object","properties":{"topic":{"type":"string","description":"Page, e.g. index."},"from":{"type":"integer","description":"Offset after next:; default 0.","minimum":0},"search":{"type":"string","description":"Up to 8 words, all required."}}}"#, effect: run::HostEffect::Read, whole: None, offer: Offer::Every },
    SEE,
    DOCUMENT,
    REMEMBER,
];

/// The complete host-tool registry, with exact schemas and write effects.
#[must_use]
pub fn tools(timeout: Duration) -> Box<[run::HostTool]> {
    offered_tools(timeout, tasks::Tools(u64::MAX), tasks::Tools(u64::MAX), true)
}

/// Each family requires both the charter and authority; asking charters keep their own tools.
/// Every non-asking run receives `read_docs` (domain/agent/declarations.md, 1).
/// Root authority still decides each accepted call by its durable name.
pub(crate) fn tools_for(timeout: Duration, charter: tasks::Tools, authority: tasks::Tools) -> Box<[run::HostTool]> {
    offered_tools(timeout, charter, authority, false)
}

fn offered_tools(
    timeout: Duration,
    charter: tasks::Tools,
    authority: tasks::Tools,
    full: bool,
) -> Box<[run::HostTool]> {
    let asking = engine::asking(charter);
    let plan = authority.0 & jig_domain_core::ToolFamilies::JIG.control.0 != 0;
    let mut tools = List::with_capacity(u32::try_from(SPECS.len()).expect("fixed tool list"));
    for spec in SPECS {
        if asking && !full && spec.name == b"propose_settings" {
            continue;
        }
        if !full && !spec.offer.allows(charter, authority) {
            continue;
        }
        tools.push(declaration(spec, timeout, plan)).expect("fixed tool list");
    }
    if asking && !full && ASK_PROPOSE_SETTINGS.offer.allows(charter, authority) {
        tools.push(declaration(ASK_PROPOSE_SETTINGS, timeout, false)).expect("fixed tool list");
    }
    tools.into_boxed()
}

pub(crate) fn declaration(spec: Spec, timeout: Duration, plan: bool) -> run::HostTool {
    let spec = if plan && spec.name == b"change" { PLAN_CHANGE } else { spec };
    run::HostTool {
        reserve: false,
        name: bytes::copy_of(spec.name),
        description: bytes::copy_of(spec.description),
        schema: bytes::copy_of(spec.schema),
        effect: spec.effect,
        timeout,
    }
}

/// Decode an attested Smith host call under one activation-qualified name.
/// The caller supplies its task/attempt envelope separately to the root.
pub fn call(name: run::CallName, tool: &[u8], input: &run::HostInput) -> Result<Decoded, Problem> {
    call_configured(name, tool, input, &[], &[])
}

/// Decode host calls with execution models and whole catalogue settings.
pub fn call_configured(
    name: run::CallName,
    tool: &[u8],
    input: &run::HostInput,
    models: &[engine::Model],
    catalogue: &[jig_domain_core::settings::CatalogueModel],
) -> Result<Decoded, Problem> {
    call_in(name, tool, input, models, catalogue, &[])
}

/// Decode host calls with the task project's readable repository identities.
#[expect(clippy::too_many_lines, reason = "one exhaustive finite host-tool registry")]
pub fn call_in(
    name: run::CallName,
    tool: &[u8],
    input: &run::HostInput,
    models: &[engine::Model],
    catalogue: &[jig_domain_core::settings::CatalogueModel],
    repositories: &[engine::RepositoryIdentity],
) -> Result<Decoded, Problem> {
    if tool == b"propose_settings" {
        return crate::settings::proposal(name, input.bytes(), catalogue, repositories);
    }
    let value = crate::json::parse(input.bytes())?;
    let tool = match tool {
        b"see" => match crate::see::decode(&value)? {
            Ok(tool) => tool,
            Err(answer) => return Ok(Decoded::Answered(answer)),
        },
        b"document" => crate::document::decode(&value)?,
        b"remember" => crate::remember::decode(&value)?,
        b"read_task" => task_read(&value)?,
        b"settings" => crate::settings::read(&value, input.bytes())?,
        b"read_docs" => {
            let answer = match docs::read(&value)? {
                Ok(read) => docs::answer(&temper_docs::PAGES, &read),
                Err(answer) => answer,
            };
            return Ok(Decoded::Answered(answer));
        }
        b"read_forge" | b"read_issues" | b"read_ci" => match crate::forge_reads::decode(tool, &value)? {
            Ok(tool) => tool,
            Err(answer) => return Ok(Decoded::Answered(answer)),
        },
        b"read_items"
        | b"read_item"
        | b"read_pull"
        | b"read_pull_for"
        | b"read_reviews"
        | b"read_statuses"
        | b"read_remarks"
        | b"read_branch"
        | b"read_branches"
        | b"read_pull_files"
        | b"read_compare"
        | b"read_checks"
        | b"read_file"
        | b"read_job"
        | b"read_protection"
        | b"read_settings"
        | b"read_collaborators"
        | b"read_permission" => {
            engine::Tool::ReadNamed { repository: forge_repository(&value)?, read: forge::read_named(tool, &value)? }
        }
        b"effect_forge" => engine::Tool::EffectForge {
            repository: forge::repository(value.required(b"repository")?)?,
            resource: forge::resource(value.required(b"resource")?)?,
            write: Box::new(forge::write(value.required(b"write")?)?),
        },
        b"subscribe_forge" => forge::subscription(
            value.required(b"topic")?,
            match value.field(b"own_change")? {
                Some(change) => Some(change.number(b"own_change")?),
                None => None,
            },
            match value.field(b"paths")? {
                Some(paths) => forge::texts(paths, b"paths")?,
                None => Box::new([]),
            },
        )?,
        b"decided" => engine::Tool::Decided(decided(&value)?),
        b"red" => engine::Tool::Red(crate::red::decode(&value)?),
        b"finding" => engine::Tool::Finding(crate::finding::decode(&value)?),
        b"follow" => engine::Tool::Follow(crate::follow::decode(&value)?),
        b"goal" => {
            let words = value.required(b"words")?.text(b"words")?;
            if words.is_empty() {
                return Err(Problem::Range(b"words"));
            }
            engine::Tool::Goal {
                words,
                advisor: optional_bool(&value, b"advisor", false)?,
                agree: optional_bool(&value, b"agree", false)?,
            }
        }
        b"change" => engine::Tool::Change { changes: change_calls(&value, models)? },
        b"delegate" => engine::Tool::Delegate { batch: nested::delegates(value.required(b"batch")?)? },
        b"amend" => amend(&value)?,
        b"message" => {
            let target = positive(&value, b"target")?;
            let words = value.required(b"words")?.text(b"words")?;
            let form = value.required(b"form")?.text(b"form")?;
            let form = match form.as_ref() {
                b"words" => engine::MessageForm::Words,
                b"question" => engine::MessageForm::Question,
                b"answer" => engine::MessageForm::Answer { question: positive(&value, b"question")? },
                _ => return Err(Problem::Range(b"form")),
            };
            engine::Tool::Message { target, form, words }
        }
        b"cancel" => engine::Tool::Cancel {
            target: positive(&value, b"target")?,
            reason: value.required(b"reason")?.text(b"reason")?,
        },
        b"release" => engine::Tool::Release {
            target: positive(&value, b"target")?,
            revision: number(&value, b"revision")?,
            again: match value.field(b"again")? {
                Some(again) => Some(again.boolean(b"again")?),
                None => None,
            },
        },
        b"introduce" => {
            engine::Tool::Introduce { left: positive(&value, b"left")?, right: positive(&value, b"right")? }
        }
        b"unsubscribe" => engine::Tool::Unsubscribe { subscription: positive(&value, b"subscription")? },
        b"withdraw" => engine::Tool::Withdraw { proposal: positive(&value, b"proposal")? },
        b"decide_escalation" => {
            let decision = value.required(b"decision")?.text(b"decision")?;
            let decision = match decision.as_ref() {
                b"release" => engine::EscalationChoice::Release,
                b"reject" => engine::EscalationChoice::Reject { reason: value.required(b"reason")?.text(b"reason")? },
                b"pass" => engine::EscalationChoice::Pass,
                _ => return Err(Problem::Range(b"decision")),
            };
            engine::Tool::DecideEscalation {
                task: positive(&value, b"task")?,
                revision: positive(&value, b"revision")?,
                decision,
            }
        }
        b"decide" => {
            let decision = value.required(b"decision")?.text(b"decision")?;
            let decision = match decision.as_ref() {
                b"accept" => engine::ProposalChoice::Accept,
                b"reject" => engine::ProposalChoice::Reject { reason: value.required(b"reason")?.text(b"reason")? },
                b"pass" => engine::ProposalChoice::Pass,
                _ => return Err(Problem::Range(b"decision")),
            };
            engine::Tool::Decide {
                proposer: positive(&value, b"proposer")?,
                proposal: positive(&value, b"proposal")?,
                decision,
            }
        }
        b"subscribe" => {
            let kind = value.required(b"kind")?.text(b"kind")?;
            let kind = match kind.as_ref() {
                b"task" => tasks::SubscriptionKind::Task {
                    target: positive(&value, b"target")?,
                    held: optional_bool(&value, b"held", false)?,
                    result: optional_bool(&value, b"result", true)?,
                },
                b"timer" => tasks::SubscriptionKind::Timer {
                    at: Wall::from_nanos(number(&value, b"at")?),
                    period: match value.field(b"period")? {
                        Some(period) => Some(Duration::from_nanos(period.number(b"period")?)),
                        None => None,
                    },
                },
                _ => return Err(Problem::Range(b"kind")),
            };
            engine::Tool::Subscribe { kind }
        }
        b"propose" => {
            let action = value.required(b"action")?.text(b"action")?;
            let action = match action.as_ref() {
                b"effect" => engine::ProposedAction::Effect {
                    repository: forge::repository(value.required(b"repository")?)?,
                    resource: forge::resource(value.required(b"resource")?)?,
                    write: Box::new(forge::write(value.required(b"write")?)?),
                },
                b"release" => engine::ProposedAction::Release {
                    task: positive(&value, b"task")?,
                    hold: positive(&value, b"hold")?,
                    revision: positive(&value, b"revision")?,
                },
                b"batch" => engine::ProposedAction::Batch(nested::delegates(value.required(b"batch")?)?),
                b"amend" => engine::ProposedAction::Amend {
                    task: positive(&value, b"task")?,
                    revision: number(&value, b"revision")?,
                    amendment: nested::amendment(value.required(b"amendment")?)?,
                },
                b"widen" => engine::ProposedAction::Widen {
                    task: positive(&value, b"task")?,
                    authority: nested::authority(value.required(b"authority")?)?,
                },
                _ => return Err(Problem::Range(b"action")),
            };
            engine::Tool::Propose {
                action,
                reason: value.required(b"reason")?.text(b"reason")?,
                as_holder: optional_bool(&value, b"as_holder", false)?,
            }
        }
        b"note" => {
            let (entry, recalled, range) = notes::new(&value)?;
            match range {
                Some(range) => {
                    engine::Tool::NoteLines { entry, recalled: recalled.expect("range names its revision"), range }
                }
                None => engine::Tool::Note { entry, recalled },
            }
        }
        b"recall" => {
            let (by, page) = notes::recall(&value)?;
            engine::Tool::Recall { by, page }
        }
        _ => return Err(Problem::UnknownTool),
    };
    Ok(Decoded::Call(engine::Call { completion: name.completion, position: name.position, tool }))
}

/// Decode the forge's bounded decision words (temper/docs/design/domain/forge/attention.md, 5).
fn decided(value: &Value) -> Result<temper_engine_domain_forge::DecidedCall, Problem> {
    use temper_engine_domain_forge as forge;
    let question = value.required(b"question")?.text(b"question")?;
    if question.len() > forge::DECIDED_QUESTION_BYTES {
        return Err(Problem::Range(b"question: at most 400 bytes"));
    }
    let items = value.required(b"options")?.array(b"options")?;
    if !forge::DECIDED_OPTIONS.contains(&items.len()) {
        return Err(Problem::Range(b"options: two to six options"));
    }
    let mut options = List::with_capacity(u32::try_from(items.len()).expect("two to six options"));
    for item in items {
        let option = item.text(b"options")?;
        if option.len() > forge::DECIDED_OPTION_BYTES {
            return Err(Problem::Range(b"options: each at most 120 bytes"));
        }
        options.push(option).expect("counted options");
    }
    let took = value.required(b"took")?.number(b"took")?;
    let took = u8::try_from(took).or(Err(Problem::Range(b"took: index into options, starting at zero")))?;
    if u32::from(took) >= options.len() {
        return Err(Problem::Range(b"took: index into options, starting at zero"));
    }
    let why = value.required(b"why")?.text(b"why")?;
    if why.len() > forge::DECIDED_WHY_BYTES {
        return Err(Problem::Range(b"why: at most 400 bytes"));
    }
    let kind = temper_protocol_reaches::Kind::named(&value.required(b"kind")?.text(b"kind")?).ok_or(Problem::Range(
        b"kind: wording, retry, implementation, tests, dependency, departure, money, irreversible, rules, other or findings",
    ))?;
    let paths = match value.field(b"paths")? {
        None => Box::new([]),
        Some(value) => {
            let items = value.array(b"paths")?;
            if items.len() > temper_protocol_reaches::PATHS {
                return Err(Problem::Range(b"paths: at most 16 paths"));
            }
            let mut paths = List::with_capacity(u32::try_from(items.len()).expect("at most 16 paths"));
            for item in items {
                let path = item.text(b"paths")?;
                if path.is_empty() {
                    return Err(Problem::Range(b"paths: a path is empty"));
                }
                if path.len() > temper_protocol_reaches::PATH_BYTES {
                    return Err(Problem::Range(b"paths: each at most 256 bytes"));
                }
                if path.starts_with(b"/") {
                    return Err(Problem::Range(b"paths: repository-relative paths, no leading /"));
                }
                paths.push(path).expect("counted paths");
            }
            paths.into_boxed()
        }
    };
    let under = match value.field(b"under")? {
        None => 0,
        Some(value) => {
            let number = value.number(b"under")?;
            if !(1..=20).contains(&number) {
                return Err(Problem::Range(b"under: item number 1 to 20"));
            }
            u8::try_from(number).expect("1 to 20")
        }
    };
    Ok(forge::DecidedCall { question, options: options.into_boxed(), took, why, kind, paths, under })
}

fn number(value: &Value, field: &'static [u8]) -> Result<u64, Problem> {
    value.required(field)?.number(field)
}

fn positive(value: &Value, field: &'static [u8]) -> Result<u64, Problem> {
    let number = number(value, field)?;
    if number == 0 { Err(Problem::Range(field)) } else { Ok(number) }
}

fn optional_bool(value: &Value, field: &'static [u8], default: bool) -> Result<bool, Problem> {
    match value.field(field)? {
        Some(value) => value.boolean(field),
        None => Ok(default),
    }
}

fn change_calls(value: &Value, models: &[engine::Model]) -> Result<Box<[engine::ChangeCall]>, Problem> {
    let Some(batch) = value.field(b"changes")? else { return Ok(Box::new([change(value, models)?])) };
    change_batch_options(value)?;
    let items = batch.array(b"changes")?;
    if items.is_empty() {
        return Err(Problem::Range(b"changes"));
    }
    let capacity = u32::try_from(items.len()).or(Err(Problem::TooLarge))?;
    let mut changes = List::with_capacity(capacity);
    for item in items {
        if changes.push(change(item.object(b"changes")?, models)?).is_err() {
            return Err(Problem::TooLarge);
        }
    }
    Ok(changes.into_boxed())
}

fn change_batch_options(value: &Value) -> Result<(), Problem> {
    if value.field(b"words")?.is_some() {
        return Err(Problem::Range(b"changes"));
    }
    for member in [
        b"repository".as_slice(),
        b"name",
        b"landing",
        b"branch",
        b"merge",
        b"after",
        b"producer",
        b"llm",
        b"repairs",
        b"questions",
    ] {
        if value.field(member)?.is_some() {
            return Err(Problem::Beside(member));
        }
    }
    Ok(())
}

#[expect(clippy::too_many_lines, reason = "each change member is decoded together at the host-tool boundary")]
fn change(value: &Value, models: &[engine::Model]) -> Result<engine::ChangeCall, Problem> {
    if value.field(b"changes")?.is_some() {
        return Err(Problem::Range(b"changes"));
    }
    let words = value.required(b"words")?.text(b"words")?;
    if words.is_empty() {
        return Err(Problem::Range(b"words"));
    }
    let repository = match value.field(b"repository")? {
        Some(name) => {
            let name = name.text(b"repository")?;
            let mut slash = None;
            for (at, byte) in name.iter().enumerate() {
                if *byte == b'/' {
                    if slash.is_some() {
                        return Err(Problem::Range(b"repository"));
                    }
                    slash = Some(at);
                }
            }
            let slash = slash.ok_or(Problem::Range(b"repository"))?;
            if slash == 0 || slash.checked_add(1) == Some(name.len()) {
                return Err(Problem::Range(b"repository"));
            }
            Some(name)
        }
        None => None,
    };
    let name = match value.field(b"name")? {
        Some(name) => {
            let name = name.text(b"name")?;
            if name.is_empty() || name.len() > 64 {
                return Err(Problem::Range(b"name"));
            }
            Some(name)
        }
        None => None,
    };
    let landing = match value.field(b"landing")? {
        Some(landing) => {
            let landing = landing.text(b"landing")?;
            if landing.is_empty() {
                return Err(Problem::Range(b"landing"));
            }
            Some(if landing.as_ref() == b"goal" {
                engine::ChangeLanding::Goal
            } else {
                engine::ChangeLanding::Branch(landing)
            })
        }
        None => None,
    };
    let branch = match value.field(b"branch")? {
        Some(branch) => {
            let branch = branch.text(b"branch")?;
            Some(match branch.as_ref() {
                b"mine" => engine::ChangeBranch::Mine,
                _ => return Err(Problem::Range(b"branch")),
            })
        }
        None => None,
    };
    let merge = match value.field(b"merge")? {
        Some(value) => Some(merge_commit(value)?),
        None => None,
    };
    let after = match value.field(b"after")? {
        Some(after) => {
            let items = after.array(b"after")?;
            let Ok(capacity) = u32::try_from(items.len()) else { return Err(Problem::TooLarge) };
            let mut after = List::with_capacity(capacity);
            for item in items {
                let dependency = match item {
                    Value::String(name) => {
                        if name.is_empty() {
                            return Err(Problem::Range(b"after"));
                        }
                        engine::ChangeAfter::Name(name.clone())
                    }
                    Value::Number(_) => {
                        let task = item.number(b"after")?;
                        if task == 0 {
                            return Err(Problem::Range(b"after"));
                        }
                        engine::ChangeAfter::Task(task)
                    }
                    Value::Object(_) | Value::Array(_) | Value::Boolean(_) | Value::Null => {
                        return Err(Problem::Type(b"after"));
                    }
                };
                if after.push(dependency).is_err() {
                    return Err(Problem::TooLarge);
                }
            }
            after.into_boxed()
        }
        None => Box::new([]),
    };
    let questions = match value.field(b"questions")? {
        Some(questions) => Some(match questions.text(b"questions")?.as_ref() {
            b"none" => engine::ChangeQuestions::None,
            b"up" => engine::ChangeQuestions::Up,
            _ => return Err(Problem::Range(b"questions")),
        }),
        None => None,
    };
    let repairs = match value.field(b"repairs")? {
        Some(value) => {
            let text = value.text(b"repairs")?;
            if text.len() != 40 && text.len() != 64 {
                return Err(Problem::Range(b"repairs"));
            }
            let mut commit = [0_u8; 32];
            for (pair, cell) in text.as_chunks::<2>().0.iter().zip(&mut commit) {
                for digit in pair {
                    let nibble = match digit {
                        b'0'..=b'9' => digit.saturating_sub(b'0'),
                        b'a'..=b'f' => digit.saturating_sub(b'a').saturating_add(10),
                        _ => return Err(Problem::Range(b"repairs")),
                    };
                    *cell = cell.saturating_mul(16).saturating_add(nibble);
                }
            }
            Some(commit)
        }
        None => None,
    };
    let producer = change_producer(value.field(b"producer")?)?;
    let llm = change_llm(value.field(b"llm")?, models)?;
    Ok(engine::ChangeCall { words, repository, name, landing, branch, merge, after, producer, llm, questions, repairs })
}

fn change_producer(value: Option<&Value>) -> Result<Option<u32>, Problem> {
    let Some(value) = value else { return Ok(None) };
    let producer = value.small(b"producer")?;
    if producer == 0 {
        return Err(Problem::Range(b"producer"));
    }
    Ok(Some(producer))
}

fn change_llm(value: Option<&Value>, models: &[engine::Model]) -> Result<engine::ChangeLlm, Problem> {
    let mut llm = engine::ChangeLlm::default();
    let Some(value) = value else { return Ok(llm) };
    let Value::Object(members) = value else { return Err(Problem::Type(b"llm")) };
    for (role, choice) in members {
        match role.as_ref() {
            b"producer" => llm.producer = Some(change_choice(choice, models)?),
            b"review" => llm.review = Some(change_choice(choice, models)?),
            _ => return Err(Problem::Range(b"llm")),
        }
    }
    Ok(llm)
}

fn change_choice(value: &Value, models: &[engine::Model]) -> Result<tasks::LlmChoice, Problem> {
    let value = value.object(b"llm")?;
    let main = change_model_effort(value.required(b"main")?, models)?;
    let subagents = match value.field(b"sub_agents")? {
        Some(values) => {
            let items = values.array(b"sub_agents")?;
            let count = u32::try_from(items.len()).or(Err(Problem::TooLarge))?;
            if count > tasks::MODELS {
                return Err(Problem::TooLarge);
            }
            let mut subagents = List::with_capacity(count);
            for item in items {
                subagents.push(change_model_effort(item, models)?).or(Err(Problem::TooLarge))?;
            }
            subagents.into_boxed()
        }
        None => Box::new([]),
    };
    Ok(tasks::LlmChoice { later: None, account: None, main, subagents })
}

fn change_model_effort(value: &Value, models: &[engine::Model]) -> Result<tasks::ModelEffort, Problem> {
    let value = value.object(b"model")?;
    let name = value.required(b"model")?.text(b"model")?;
    for (index, model) in models.iter().enumerate() {
        if model.name != name {
            continue;
        }
        let effort = match value.field(b"effort")? {
            Some(value) => {
                let name = value.text(b"effort")?;
                let mut found = None;
                for (index, effort) in model.efforts.iter().enumerate() {
                    if *effort == name {
                        found = Some(u8::try_from(index).or(Err(Problem::Range(b"effort")))?);
                        break;
                    }
                }
                Some(found.ok_or(Problem::Range(b"effort"))?)
            }
            None => None,
        };
        return Ok(tasks::ModelEffort { model: u8::try_from(index).or(Err(Problem::Range(b"model")))?, effort });
    }
    Err(Problem::Range(b"model"))
}

// A catch-up accepts either hexadecimal case; procedure parameters use lowercase.
fn merge_commit(value: &Value) -> Result<[u8; 32], Problem> {
    let text = value.text(b"merge")?;
    if text.len() != 40 && text.len() != 64 {
        return Err(Problem::Range(b"merge"));
    }
    let mut commit = [0_u8; 32];
    for (pair, cell) in text.as_chunks::<2>().0.iter().zip(&mut commit) {
        let mut byte = 0_u8;
        for digit in pair {
            let digit = match digit.to_ascii_lowercase() {
                digit @ b'0'..=b'9' => digit.saturating_sub(b'0'),
                digit @ b'a'..=b'f' => digit.saturating_sub(b'a').saturating_add(10),
                _ => return Err(Problem::Range(b"merge")),
            };
            byte = byte.saturating_mul(16).saturating_add(digit);
        }
        *cell = byte;
    }
    Ok(commit)
}

fn amend(value: &Value) -> Result<engine::Tool, Problem> {
    let amendment = nested::amendment(value.required(b"amendment")?)?;
    let keep_parameters = nested::keeps_parameters(value.required(b"amendment")?)?;
    let priority = match amendment.tracked {
        Some(tasks::Tracking::Tracked { .. }) => true,
        Some(tasks::Tracking::Untracked) | None => false,
    };
    let revision = match value.field(b"revision")? {
        Some(value) => value.number(b"revision")?,
        None if priority
            && amendment.spec.is_none()
            && amendment.wake.is_none()
            && amendment.dependencies.is_none()
            && amendment.authority.is_none()
            && amendment.instructions.is_none()
            && amendment.llm.is_none() =>
        {
            0
        }
        None => return Err(Problem::Missing(b"revision")),
    };
    Ok(engine::Tool::Amend { keep_parameters, target: positive(value, b"target")?, revision, amendment })
}

fn task_read_field(value: &Value) -> Result<jig_domain_core::reads::TaskTextField, Problem> {
    use jig_domain_core::reads::TaskTextField;
    match value.field(b"field")? {
        None => Ok(TaskTextField::Specification),
        Some(field) => match field.text(b"field")?.as_ref() {
            b"specification" => Ok(TaskTextField::Specification),
            b"result" => Ok(TaskTextField::Result),
            _ => Err(Problem::Range(b"field")),
        },
    }
}

fn task_read(value: &Value) -> Result<engine::Tool, Problem> {
    Ok(engine::Tool::ReadTask {
        field: task_read_field(value)?,
        task: positive(value, b"task")?,
        from: match value.field(b"from")? {
            Some(from) => from.number(b"from")?,
            None => 0,
        },
    })
}

pub(crate) fn forge_repository(value: &Value) -> Result<Option<Box<[u8]>>, Problem> {
    match value.field(b"repository")? {
        Some(value) => {
            let name = value.text(b"repository")?;
            if name.is_empty() { Ok(None) } else { Ok(Some(name)) }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
#[path = "tests/declarations.rs"]
mod tests;
