use super::*;
use skein_ui::{Role, Run};
fn text(b: &mut skein_ui::Builder, role: Role, words: &[u8]) {
    b.open(role);
    b.text(Run::Plain, words);
    b.close();
}
fn region(b: &mut skein_ui::Builder, name: &[u8]) {
    b.open(Role::Region);
    b.name(name);
    text(b, Role::Heading { level: 2 }, name);
}
fn chip(b: &mut skein_ui::Builder, name: &[u8], title: &[u8], phase: &[u8], spent: &[u8]) {
    b.open(Role::TreeItem);
    b.name(name);
    b.open(Role::Slot);
    b.style(temper_client_view::look::PLAN_ROW);
    b.open(Role::Slot);
    b.open(Role::Link);
    b.name(name);
    b.bind(binding());
    b.close();
    text(b, Role::Paragraph, title);
    b.close();
    text(b, Role::Paragraph, b"Agent");
    text(b, Role::Status, phase);
    b.open(Role::Paragraph);
    b.style(temper_client_view::look::PLAN_SPEND);
    b.name(spent);
    b.text(Run::Plain, spent);
    b.close();
    b.close();
}
pub fn page() -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    text(&mut b, Role::Heading { level: 1 }, b"Refresh token handling");
    region(&mut b, b"Task details");
    for words in [b"Task 9".as_slice(), b"Refresh token handling", b"Agent"] {
        text(&mut b, Role::Paragraph, words);
    }
    text(&mut b, Role::Status, b"running");
    for words in [b"Shop".as_slice(), b"Asked by Pat", "Goal · priority 1".as_bytes()] {
        text(&mut b, Role::Paragraph, words);
    }
    b.close();
    b.open(Role::Slot);
    b.style(temper_client_view::look::TASK_ACTS);
    b.open(Role::Button);
    b.name(b"Amend");
    b.bind(binding());
    b.close();
    b.close();
    region(&mut b, b"What was asked");
    text(&mut b, Role::Paragraph, "Result · report".as_bytes());
    text(&mut b, Role::Paragraph, b"Rotate tokens safely.");
    text(&mut b, Role::Paragraph, b"Shown on the forge.");
    b.close();
    region(&mut b, b"Plan");
    text(&mut b, Role::Paragraph, b"Its delegates: 2 live, 1 ended.");
    b.open(Role::Slot);
    b.style(temper_client_view::look::FILTERS);
    for name in [b"All".as_slice(), b"Live only"] {
        b.open(Role::Button);
        b.name(name);
        if name == b"All" {
            b.state(skein_ui::State::Pressed);
        }
        b.bind(binding());
        b.close();
    }
    b.close();
    b.open(Role::Tree);
    b.name(b"Plan");
    chip(&mut b, b"Task 11", b"Advise on the change", b"Completed", b"3 spent");
    b.style(temper_client_view::look::PLAN_DONE);
    b.close();
    chip(&mut b, b"Task 12", b"Review the change", b"Held", b"4 spent");
    golden_cards::held(&mut b);
    b.close();
    chip(&mut b, b"Task 13", b"Write the release notes", b"waiting on task 12", b"0 spent");
    b.open(Role::List);
    b.name(b"Dependencies of task 13");
    b.style(temper_client_view::look::DEPENDENCIES);
    b.open(Role::Item);
    b.key(skein_ui::Key { owner: temper_client_view::frame::owner(), kind: 0, a: 12, b: 0 });
    b.open(Role::Link);
    b.name(b"Task 12");
    b.text(Run::Plain, b"Task 12");
    b.bind(binding());
    b.close();
    text(&mut b, Role::Status, b"Held");
    b.close();
    b.close();
    b.close();
    text(&mut b, Role::Status, b"Held below: 1");
    b.close();
    b.close();
    region(&mut b, b"Latest activity");
    b.open(Role::Item);
    text(&mut b, Role::Paragraph, b"Jan 1, 00:00");
    text(&mut b, Role::Paragraph, b"The review needs attention.");
    b.close();
    b.close();
    region(&mut b, b"Budget and authority");
    text(&mut b, Role::Paragraph, "7 spent · 493 left of 500".as_bytes());
    for (term, description) in [
        (b"Delegates spent".as_slice(), b"0".as_slice()),
        (b"Reserved for delegates", b"0"),
        (b"Funded by", b"Your pool"),
        (b"Deadline", b"None"),
        (b"May spend", b"500"),
        (b"May delegate", b"Up to 1 task, 1 deep"),
        (b"Models", b"All"),
    ] {
        text(&mut b, Role::Term, term);
        text(&mut b, Role::Description, description);
    }
    b.close();
    b.close();
    b.close();
    b.finish()
}

/// A live delegate's brief and conversation (client/README.md, 5.9).
pub fn agent_task_page() -> skein_ui::Tree {
    use temper_client_view::look;
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::CHAT);
    b.open(Role::Slot);
    b.style(look::AGENT_LINEAGE);
    b.open(Role::Slot);
    task_link(&mut b, b"Task 12");
    text(&mut b, Role::Paragraph, b"Rotate the credentials");
    b.close();
    b.open(Role::Slot);
    text(&mut b, Role::Paragraph, b"Task 34");
    text(&mut b, Role::Paragraph, b"Refresh token handling");
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::CHATBAR);
    b.open(Role::Slot);
    text(&mut b, Role::Heading { level: 1 }, "T34 · Refresh token handling".as_bytes());
    b.open(Role::Status);
    b.style(look::STATUS_ACTIVE);
    b.text(Run::Plain, "running · writing".as_bytes());
    b.close();
    b.close();
    b.open(Role::Slot);
    text(&mut b, Role::Paragraph, b"Agent");
    text(&mut b, Role::Paragraph, b"7 spent");
    b.open(Role::Button);
    b.name(b"Amend");
    b.bind(binding());
    b.close();
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::THREAD);
    region(&mut b, b"Task brief");
    b.style(look::AGENT_BRIEF);
    skein_ui::markdown::read(b"Rotate tokens **safely**.", 3, &mut b);
    text(&mut b, Role::Paragraph, b"Asked by task 12");
    b.open(Role::List);
    b.name(b"Dependencies of task 34");
    b.style(look::DEPENDENCIES);
    b.open(Role::Item);
    b.key(skein_ui::Key { owner: temper_client_view::frame::owner(), kind: 0, a: 17, b: 0 });
    task_link(&mut b, b"Task 17");
    b.open(Role::Status);
    b.style(look::STATUS_DONE);
    b.text(Run::Plain, b"Completed");
    b.close();
    b.close();
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::AGENT_RUN);
    text(&mut b, Role::Heading { level: 2 }, "Run 1 · live now".as_bytes());
    text(&mut b, Role::Paragraph, b"Started 10-22 03:48Z");
    b.close();
    b.open(Role::Log);
    b.name(b"Conversation");
    b.open(Role::Region);
    b.style(look::TURN);
    b.open(Role::Slot);
    b.style(look::MESSAGE);
    b.style(look::BY_AGENT);
    b.open(Role::Slot);
    b.style(look::WHO);
    text(&mut b, Role::Paragraph, b"Temper");
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    skein_ui::markdown::read(b"Checking the credentials.", 3, &mut b);
    b.open(Role::Disclosure);
    b.style(look::WORK);
    b.name(b"1 tool call");
    b.description(b"read_file");
    b.open(Role::Slot);
    b.style(look::WORK);
    text(&mut b, Role::Paragraph, b"read_file");
    text(&mut b, Role::Paragraph, b"Read token.rs");
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.close();
    b.finish()
}
fn task_link(b: &mut skein_ui::Builder, name: &[u8]) {
    b.open(Role::Link);
    b.name(name);
    b.text(Run::Plain, name);
    b.bind(binding());
    b.close();
}
