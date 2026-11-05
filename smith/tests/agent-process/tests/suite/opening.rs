//! One person's brief shared by direct, program and seeded Open stories.
use skein_lib::{Duration, List};
use smith_charter::{CEILINGS, Mode, Section, SectionParts, Tools};

pub const INSTRUCTIONS: &[u8] = b"Coordinate the work; ask before changing files.";
pub const BRIEF: &str = "## Request\n\nRead café; propose a plan and ask for my go-ahead.\n\n## Context\n\nKeep the person's words whole.\n\n";

// Two literal sections; ordinary turns/spend match the existing program-tools fixture.
const BRIEF_SECTIONS: u32 = 2;
const TURNS: u32 = 8;
const SPEND: u64 = 20;

pub fn charter() -> Box<[u8]> {
    smith_agent_process_world::wire::edit_charter(&smith_agent_process_world::charter(), |parts| {
        parts.mode = Mode::Open;
        parts.instructions = INSTRUCTIONS.into();
        parts.waiting = Duration::from_millis(10);
        let mut budget = parts.budget.clone().into_parts();
        budget.turns = TURNS;
        budget.spend = SPEND;
        parts.budget = smith_charter::Budget::new(&CEILINGS, budget).expect("chat budget");
        parts.brief = List::with_capacity(BRIEF_SECTIONS);
        for (title, text) in [
            ("Request", "Read café; propose a plan and ask for my go-ahead."),
            ("Context", "Keep the person's words whole."),
        ] {
            parts
                .brief
                .push(
                    Section::new(
                        &CEILINGS,
                        SectionParts { title: title.as_bytes().into(), body: text.as_bytes().into() },
                    )
                    .expect("brief section"),
                )
                .expect("two sections");
        }
        let mut tools = parts.tools.clone().into_parts();
        tools.wait = true;
        tools.deliver = None;
        parts.tools = Tools::new(&CEILINGS, tools).expect("Open tools");
    })
}
