//! A setting read previews a new task's charter (jig/docs/design/domain/engine/project-charters.md, 7).
use crate::reads::{Answer, ProjectCharterDocument, Read, SettingDocument, Why};
use crate::settings as s;
use crate::tests::project_charters::{NAME, binding::*};
use crate::tests::*;

const SIGN_IN: u64 = 10;
const SET_REVISION: u64 = 4;
const IMPLEMENTER: u32 = 2;
const CAREFUL: u32 = 33;
const UNNAMED: u32 = 1;
const NO_SET: u32 = 1;
const CAREFUL_NAME: &[u8] = b"careful";
const MISSING: &[u8] = b"missing";
const OVERLAY_WORDS: &[u8] = b"Keep notes.";
const SOURCE: &[u8] = b"application source";

fn fixture() -> (Core, Env<Limits>) {
    fixture_with_roles(&[], &[people::DEPLOYMENT, NO_SET, PROJECT])
}

fn setting_read(number: u32, revision: u32, project: Option<u32>) -> Read {
    Read::Setting { setting: s::Setting::Charter(number), revision, proposal: None, project }
}

fn read(core: &mut Core, env: &Env<Limits>, request: Read) -> Answer {
    let Held::Read { answer, .. } = read_start(core, &env.limits, SIGN_IN, request) else {
        panic!("terminal setting read");
    };
    answer
}

fn document(core: &mut Core, env: &Env<Limits>, request: Read) -> SettingDocument {
    let answer = read(core, env, request);
    let Answer::Setting(document) = answer else { panic!("expected setting document, got {answer:?}") };
    *document
}

fn project_document(settings: &[u8]) -> ProjectCharterDocument {
    ProjectCharterDocument {
        name: NAME.into(),
        revision: SET_REVISION,
        source: SOURCE.into(),
        settings: settings.into(),
        instructions: PROJECT_WORDS.into(),
        parts: Box::new([CharterPart { on: tasks::Reasons::FIRST, instructions: FIRST_PART.into() }]),
    }
}

fn set_overlay(core: &mut Core, env: &Env<Limits>, number: u32) -> (u32, s::Overlay) {
    let overlay = s::Overlay {
        instructions: OVERLAY_WORDS.into(),
        parts: Box::new([CharterPart {
            on: tasks::Reasons::QUESTION,
            instructions: b"Check the answer.".as_slice().into(),
        }]),
        ..s::Overlay::empty()
    };
    let revision = change_charter(core, env, number, s::Value::Overlay(Box::new(overlay.clone())), s::Running::Stays);
    (revision, overlay)
}

#[test]
fn a_project_read_uses_the_bound_setting_and_newest_set() {
    for (settings, effective) in [(NAME, IMPLEMENTER), (CAREFUL_NAME, CAREFUL)] {
        let (mut core, env) = fixture();
        take(&mut core, &env, SET_REVISION - 1, MISSING, b"Older project instructions");
        take(&mut core, &env, SET_REVISION, settings, PROJECT_WORDS);
        let (revision, overlay) = set_overlay(&mut core, &env, effective);
        let document = document(&mut core, &env, setting_read(IMPLEMENTER, 0, Some(PROJECT)));
        let Some(s::Value::Charter(charter)) = document.value.as_ref() else { panic!("whole deployment charter") };
        assert_eq!(charter.policy.instructions.as_ref(), b"Deployment instructions.\n\nKeep notes.");
        assert_eq!(charter.name.as_deref(), Some(settings));
        assert_eq!(document.setting, s::Setting::Charter(effective));
        assert_eq!(document.revision, revision);
        assert_eq!(document.overlay.as_deref(), Some(&overlay));
        assert_eq!(document.project, Some(Box::new(project_document(settings))));
    }
}

#[test]
fn unbound_project_reads_return_the_ordinary_newest_charter() {
    let (mut core, env) = fixture();
    take(&mut core, &env, SET_REVISION, NAME, PROJECT_WORDS);
    let revision = core.current.newest();
    for (number, project) in [(CAREFUL, PROJECT), (IMPLEMENTER, NO_SET), (UNNAMED, PROJECT)] {
        let expected = read(&mut core, &env, setting_read(number, revision, None));
        assert_eq!(read(&mut core, &env, setting_read(number, 0, Some(project))), expected);
    }
}

#[test]
fn refused_bindings_keep_the_requested_value_and_project_words() {
    const PROJECT_AND_OVERLAY_PARTS: u32 = 2;
    for (settings, part_limit) in [(MISSING, PROJECT_AND_OVERLAY_PARTS), (CAREFUL_NAME, PROJECT_AND_OVERLAY_PARTS - 1)]
    {
        let (mut core, mut env) = fixture();
        let (revision, _) = set_overlay(&mut core, &env, CAREFUL);
        take(&mut core, &env, SET_REVISION, settings, PROJECT_WORDS);
        env.limits.tasks.charter_parts = part_limit;
        let mut expected = document(&mut core, &env, setting_read(IMPLEMENTER, revision, None));
        expected.project = Some(Box::new(project_document(settings)));
        assert_eq!(
            read(&mut core, &env, setting_read(IMPLEMENTER, 0, Some(PROJECT))),
            Answer::Setting(Box::new(expected))
        );
    }
}

#[test]
fn a_project_preview_refuses_incompatible_setting_read_fields() {
    let (mut core, env) = fixture();
    let revision = core.current.newest();
    for request in [
        setting_read(IMPLEMENTER, revision, Some(PROJECT)),
        Read::Setting {
            setting: s::Setting::Charter(IMPLEMENTER),
            revision: 0,
            proposal: Some((Proposer::Party(1), 1)),
            project: Some(PROJECT),
        },
        Read::Setting { setting: s::Setting::Rules, revision: 0, proposal: None, project: Some(PROJECT) },
    ] {
        assert_eq!(read(&mut core, &env, request), Answer::None(Why::Malformed));
    }
}

#[test]
fn a_project_preview_requires_both_read_audiences() {
    for roles in [[people::DEPLOYMENT], [PROJECT]] {
        let (mut core, env) = fixture_with_roles(&[], &roles);
        take(&mut core, &env, SET_REVISION, NAME, PROJECT_WORDS);
        assert_eq!(read(&mut core, &env, setting_read(IMPLEMENTER, 0, Some(PROJECT))), Answer::None(Why::Unknown));
    }
}
