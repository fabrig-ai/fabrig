//! The Goals page's terminal vocabulary (client/README.md, 3.5, Goals).
use super::{binding, synthetic_frame, tree_limits};
use skein_ui::{Builder, Role, Run, Tree};
use temper_client_view::look;

fn text(b: &mut Builder, role: Role, style: skein_ui::Style, words: &[u8]) {
    b.open(role);
    b.style(style);
    b.text(Run::Plain, words);
    b.close();
}

pub fn page() -> Tree {
    let mut b = Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::GOALS);
    b.open(Role::Slot);
    b.style(look::GOALS_INTRO);
    text(&mut b, Role::Paragraph, look::GOALS_KICKER, "Shop · goals".as_bytes());
    text(&mut b, Role::Heading { level: 1 }, look::GOALS_HEADLINE, b"More than 5 goals, 3 under way.");
    text(
        &mut b,
        Role::Paragraph,
        look::GOALS_LEDE,
        b"A goal turns a draft into landed work, its own way: task by task, plan, tasks, land whole.",
    );
    b.close();
    b.open(Role::Slot);
    b.style(look::GOALS_SECTION);
    b.open(Role::Slot);
    b.style(look::GOALS_SECTION_HEAD);
    b.open(Role::Slot);
    b.style(look::GOALS_FILTERS);
    for name in [b"All".as_slice(), b"Needs you", b"Under way", b"Ended"] {
        b.open(Role::Button);
        b.name(name);
        if name == b"All" {
            b.state(skein_ui::State::Pressed);
        }
        b.bind(binding());
        b.close();
    }
    b.close();
    b.open(Role::Link);
    b.style(look::SHEET_LINK);
    b.name(b"Start one from a draft");
    b.link(binding());
    b.text(Run::Plain, b"Start one from a draft");
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::GOALS_ROWS);
    for (name, headline, ended) in [
        ("OAuth login · 6 / 9 · needs you", "Choose the provider.", false),
        ("Refresh token handling · 2 / 4 · running", "Two tasks landed; the rest runs.", false),
        ("Write release notes · planning", "Planning from the draft.", false),
        ("Keyboard navigation · 3 / 3 · landed 01:12", "The keyboard pass landed.", true),
        ("Retire password login · closed", "Closed by you.", true),
    ] {
        b.open(Role::Slot);
        b.open(Role::Link);
        b.style(look::GOALS_ROW);
        if ended {
            b.style(look::GOALS_ROW_ENDED);
        }
        b.name(name.as_bytes());
        b.link(binding());
        b.text(Run::Plain, name.as_bytes());
        b.close();
        text(&mut b, Role::Paragraph, look::GOALS_LINE, headline.as_bytes());
        b.close();
    }
    b.close();
    b.open(Role::Button);
    b.style(look::GOALS_MORE);
    b.name(b"More goals");
    b.bind(binding());
    b.close();
    b.close();
    b.close();
    b.close();
    b.finish()
}
