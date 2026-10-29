//! Limit representation and Markdown workspace bounds (programming-model.md, 6.3).
use skein_lib::Rng;
use skein_ui::markdown::markdown_bound;
use skein_ui::phrases::{self, PhraseLimits};
use skein_ui::{Limits, valid};

// Zero capacities represent an empty UI, with no scene or Markdown scratch.
const EMPTY: Limits = Limits {
    nodes: 0,
    depth: 0,
    text_per_node: 0,
    text_bytes: 0,
    drawing_bytes: 0,
    styles_per_node: 0,
    patches: 0,
    bindings: 0,
    slots: 0,
    markdown_depth: 0,
    scene_marks: 0,
    scene_bytes: 0,
};
const EMPTY_PHRASES: PhraseLimits =
    PhraseLimits { languages: 0, phrases: 0, holes: 0, segments: 0, text_bytes: 0, hole_bytes: 0 };
// A few thousand deterministic draws for each function, as this package has no fuzzy suite.
const DRAWS: u32 = 4096;

fn draw(rng: &mut Rng, maximum: u32) -> u32 {
    u32::try_from(rng.between(0, u64::from(maximum))).expect("draw fits u32")
}

#[test]
fn valid_checks_ui_capacity_edges() {
    // Two styles per scene mark fit through floor(u32::MAX/2).
    const LAST_TWO_STYLE_MARKS: u32 = 2_147_483_647;
    let cases = [
        ("zero", EMPTY, true),
        ("max nodes", Limits { nodes: u32::MAX, ..EMPTY }, true),
        ("max depth", Limits { depth: u32::MAX, ..EMPTY }, true),
        ("max text per node", Limits { text_per_node: u32::MAX, ..EMPTY }, true),
        ("max text bytes", Limits { text_bytes: u32::MAX, ..EMPTY }, true),
        ("max drawings", Limits { drawing_bytes: u32::MAX, ..EMPTY }, true),
        ("max styles", Limits { styles_per_node: u32::MAX, ..EMPTY }, true),
        ("max patches", Limits { patches: u32::MAX, ..EMPTY }, true),
        ("max bindings", Limits { bindings: u32::MAX, ..EMPTY }, true),
        ("max slots", Limits { slots: u32::MAX, ..EMPTY }, true),
        ("max markdown depth", Limits { markdown_depth: u32::MAX, ..EMPTY }, true),
        ("max scene marks", Limits { scene_marks: u32::MAX, ..EMPTY }, true),
        ("max scene bytes", Limits { scene_bytes: u32::MAX, ..EMPTY }, true),
        ("last scene styles", Limits { scene_marks: LAST_TWO_STYLE_MARKS, styles_per_node: 2, ..EMPTY }, true),
        (
            "first scene style overflow",
            Limits { scene_marks: LAST_TWO_STYLE_MARKS + 1, styles_per_node: 2, ..EMPTY },
            false,
        ),
    ];
    for (name, limits, expected) in cases {
        assert_eq!(valid(&limits), expected, "{name}: {limits:?}");
    }
}

#[test]
fn valid_accepts_large_independent_node_and_drawing_capacities() {
    const TWO_TO_31: u32 = 2_147_483_648;
    let limits = Limits { nodes: TWO_TO_31, drawing_bytes: TWO_TO_31, ..EMPTY };
    assert!(valid(&limits));
}

#[test]
fn valid_checks_scene_style_capacity_over_seeded_ui_limits() {
    // Every field spans its type, including the scene's derived-capacity overflow cases.
    let mut rng = Rng::new(0x7067_7661_6c69_6402);
    for draw_index in 0..DRAWS {
        let limits = Limits {
            nodes: draw(&mut rng, u32::MAX),
            depth: draw(&mut rng, u32::MAX),
            text_per_node: draw(&mut rng, u32::MAX),
            text_bytes: draw(&mut rng, u32::MAX),
            drawing_bytes: draw(&mut rng, u32::MAX),
            styles_per_node: draw(&mut rng, u32::MAX),
            patches: draw(&mut rng, u32::MAX),
            bindings: draw(&mut rng, u32::MAX),
            slots: draw(&mut rng, u32::MAX),
            markdown_depth: draw(&mut rng, u32::MAX),
            scene_marks: draw(&mut rng, if draw_index % 2 == 0 { 1 } else { u32::MAX }),
            scene_bytes: draw(&mut rng, u32::MAX),
        };
        let styles = u64::from(limits.scene_marks) * u64::from(limits.styles_per_node);
        assert_eq!(valid(&limits), u32::try_from(styles).is_ok(), "draw {draw_index}: {limits:?}");
    }
}

#[test]
fn phrase_valid_checks_capacity_edges() {
    // Two plural forms per phrase fit through floor(u32::MAX/2).
    const LAST_PHRASES: u32 = 2_147_483_647;
    let cases = [
        ("zero", EMPTY_PHRASES, true),
        ("max languages", PhraseLimits { languages: u16::MAX, ..EMPTY_PHRASES }, true),
        ("max phrases", PhraseLimits { phrases: u32::MAX, ..EMPTY_PHRASES }, false),
        ("max holes", PhraseLimits { holes: u16::MAX, ..EMPTY_PHRASES }, true),
        ("max segments", PhraseLimits { segments: u32::MAX, ..EMPTY_PHRASES }, true),
        ("max text", PhraseLimits { text_bytes: u32::MAX, ..EMPTY_PHRASES }, true),
        ("max hole bytes", PhraseLimits { hole_bytes: u32::MAX, ..EMPTY_PHRASES }, true),
        ("last forms", PhraseLimits { phrases: LAST_PHRASES, ..EMPTY_PHRASES }, true),
        ("first form overflow", PhraseLimits { phrases: LAST_PHRASES + 1, ..EMPTY_PHRASES }, false),
        (
            "largest representable envelope",
            PhraseLimits {
                languages: u16::MAX,
                phrases: LAST_PHRASES,
                holes: u16::MAX,
                segments: u32::MAX,
                text_bytes: u32::MAX,
                hole_bytes: u32::MAX,
            },
            true,
        ),
    ];
    for (name, limits, expected) in cases {
        assert_eq!(phrases::valid(&limits), expected, "{name}: {limits:?}");
    }
}

#[test]
fn phrase_valid_checks_form_capacity_over_seeded_limits() {
    let mut rng = Rng::new(0x7067_7661_6c69_6403);
    for draw_index in 0..DRAWS {
        let limits = PhraseLimits {
            languages: u16::try_from(draw(&mut rng, u32::from(u16::MAX))).expect("draw fits u16"),
            phrases: draw(&mut rng, u32::MAX),
            holes: u16::try_from(draw(&mut rng, u32::from(u16::MAX))).expect("draw fits u16"),
            segments: draw(&mut rng, u32::MAX),
            text_bytes: draw(&mut rng, u32::MAX),
            hole_bytes: draw(&mut rng, u32::MAX),
        };
        assert_eq!(phrases::valid(&limits), limits.phrases <= u32::MAX / 2, "draw {draw_index}: {limits:?}");
    }
}

#[test]
fn markdown_workspace_is_representable_at_capacity_edges() {
    for (bytes, depth) in [(0, 0), (u32::MAX, 0), (0, u32::MAX), (u32::MAX, u32::MAX)] {
        assert!(markdown_bound(bytes, depth).is_some());
    }
}
