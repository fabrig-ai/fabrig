//! A document survives the note row, scoped read and recall (domain/engine.md, 10).
use super::{World, action, core, note_setup, notes};
use jig_core_world::reads::{Parties, ask, latest};
use jig_domain_core::reads;
use skein_lib::List;

#[test]
fn a_documents_whole_body_reads_back() {
    // jig/docs/design/domain/engine.md, 10: standard's document body.
    const BODY_BYTES: usize = 65_536;
    // A box's generic document text can be smaller than a note body.
    const BOX_TEXT_BYTES: u32 = 128;
    let standard = jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core;
    let body = vec![b'x'; BODY_BYTES];
    let description = vec![b't'; standard.notes.description_bytes.try_into().expect("description size")];
    // Standard and a box's smaller generic text bound both carry the note whole.
    for text_bytes in [standard.documents.text_bytes, BOX_TEXT_BYTES] {
        let mut setup = note_setup(129);
        setup.limits.core.notes.body_bytes = standard.notes.body_bytes;
        setup.limits.core.notes.description_bytes = standard.notes.description_bytes;
        setup.limits.core.call_answer_bytes = standard.call_answer_bytes;
        setup.limits.core.load_bytes = standard.load_bytes;
        setup.limits.core.documents.text_bytes = text_bytes;
        let mut world = World::new(setup);
        world.reads = Some(Parties::new(world.store.applied));
        let key = world.call_key(1);
        action(
            &mut world,
            1,
            core::NamedAction::Note {
                entry: notes::New {
                    name: 0,
                    scope: notes::Scope::Project { project: 1 },
                    description: description.as_slice().into(),
                    body: body.as_slice().into(),
                    references: List::with_capacity(0),
                    author: notes::Author::Task { task: key.task, attempt: key.attempt },
                },
                recalled: None,
            },
        );
        let core::CallPart::NoteWritten { name, revision: 1, .. } = world.answers.last().expect("written answer").2
        else {
            panic!("document write was not admitted");
        };
        ask(
            &mut world,
            1,
            reads::Read::ScopedNotes { scope: notes::Scope::Project { project: 1 }, description: None, from: None },
        );
        let reads::Answer::Page(page) = latest(&world) else { panic!("scoped notes page") };
        assert_eq!(page.rows.len(), 1, "whole note row fits the load");
        let reads::PageRow::Note(note) = page.rows.get(0).expect("whole note row") else { panic!("note document") };
        assert_eq!(note.body.bytes.len(), BODY_BYTES, "whole document body");
        assert_eq!(note.body.bytes.as_ref(), body);
        assert!(!note.body.cut);
        assert_eq!(note.description.bytes.as_ref(), description);
        assert!(!note.description.cut);
        assert!(page.next.is_none());
        action(&mut world, 2, core::NamedAction::Recall { by: notes::Recall::Name { name }, page: 0 });
        let core::CallPart::NoteRecalled { entries, more: false } = &world.answers.last().expect("recall answer").2
        else {
            panic!("document recall was not answered");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].body.as_ref(), body);
    }
}
