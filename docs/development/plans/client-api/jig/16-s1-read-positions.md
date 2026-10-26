# Supplement jig-16.s1: a conversation's read position named for no shape

Added by ruling 305 (temper-04.3's block of 2026-10-18T03:31:34Z). Read
first: jig's `AGENTS.md`, `docs/development/workflow.md`; this plan's
README, "Rules of this plan"; `code-organization.md`, 6.

Scope: jig 16 left the people child's read position over one
conversation, and the core's load of it, named for a chat. The design
names it for the flag: "its read positions (one over results, one per
task flagged conversation)" (jig's `docs/design/domain/README.md`, the
inbox across projects); code-organization.md, 6: no "record or index
named for a chat". A rename, nothing else:
- `jig_domain_people::Key::ReadChat` and `Stored::ReadChat` become
  `ReadConversation`, beside `ReadResults`;
- `jig_domain_core::Now::SeenChat` becomes `SeenConversation`, and
  `Event::SeenChatLoaded` becomes `SeenConversationLoaded`;
- `crates/jig-protocol-store/schema/people-v3.schema:46-47`: `record
  SeenChat` becomes `SeenConversation`, and `enum Seen`'s `chat:` arm
  `conversation:`; its generated code regenerated.
Needs: nothing.

Files: what `git grep -l -w -E 'ReadChat|SeenChat|SeenChatLoaded' --
'*.rs' ':!*/generated/*'` lists at the start (50 files at jig
`6605e0564`, `examples/ops/domain/src/route.rs` among them), the schema,
and its generated code. Rename any further identifier naming this read
position for a chat (a function, a fixture, a doc comment), and list it
in the commit body.

Proof: the gate. At the end, the grep above prints nothing, and `git
grep -n -w -E 'ReadChat|SeenChat' -- '*.schema'` prints nothing.
Size: a rename; aim for at most 200 changed lines outside generated code.

Don't: change a behaviour, an encoding or a test's expected value; a
test whose expected bytes would change is a question in the report, not
an edit.
