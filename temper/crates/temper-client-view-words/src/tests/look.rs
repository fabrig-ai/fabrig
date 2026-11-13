use crate::look::*;
use skein_ui::Role;

#[test]
fn the_looks_names_pass_skeins_check() {
    assert!(skein_ui::look::check(&[NAMES]).is_none());
    for (token, name) in [
        (WHY_OFFERS, b"temper-why-offers".as_slice()),
        (TOP, b"temper-top".as_slice()),
        (TOP_END, b"temper-top-end".as_slice()),
        (OUTSIDE, b"temper-outside".as_slice()),
        (OUTSIDE_PRIMARY, b"temper-outside-primary".as_slice()),
        (OUTSIDE_SECONDARY, b"temper-outside-secondary".as_slice()),
        (OUTSIDE_ACTS, b"temper-outside-acts".as_slice()),
        (OUTSIDE_QUIET, b"temper-outside-quiet".as_slice()),
        (APPROVE_FIELD, b"temper-approve-field".as_slice()),
        (PUBLIC_PAGE, b"temper-public-page".as_slice()),
        (PUBLIC_INTRO, b"temper-public-intro".as_slice()),
        (PUBLIC_LEDE, b"temper-public-lede".as_slice()),
        (PUBLIC_ROWS, b"temper-public-rows".as_slice()),
        (PUBLIC_ROW, b"temper-public-row".as_slice()),
        (PUBLIC_LINE, b"temper-public-line".as_slice()),
        (PUBLIC_FACTS, b"temper-public-facts".as_slice()),
        (PUBLIC_SECTION, b"temper-public-section".as_slice()),
        (CHAT_GOAL, b"temper-chat-goal".as_slice()),
        (WRITING_END, b"temper-writing-end"),
        (WELCOME_COPY, b"temper-welcome-copy"),
        (WELCOME_CMD_BOX, b"temper-welcome-cmd-box"),
        (CONFIRM, b"temper-confirm"),
        (CONFIRM_FATES, b"temper-confirm-fates"),
        (CONFIRM_ACTS, b"temper-confirm-acts"),
        (CONFIRM_DO, b"temper-confirm-do"),
        (RAIL_SEP, b"temper-rail-sep"),
        (WORK_WAY, b"temper-work-way"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests scan the browser stylesheet as UTF-8 text"
)]
fn every_class_the_stylesheet_selects_is_a_look_name() {
    let mut source = include_str!("../../../temper-client-shell-web/temper.css");
    let mut css = String::new();
    while let Some((before, comment)) = source.split_once("/*") {
        css.push_str(before);
        let (_, after) = comment.split_once("*/").expect("stylesheet comment ends");
        source = after;
    }
    css.push_str(source);
    let mut classes = alloc::collections::BTreeSet::new();
    for block in css.split('{') {
        let selector = block.rsplit('}').next().expect("selector before declarations");
        for class in selector.split(".temper-").skip(1) {
            let end = class.find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_').unwrap_or(class.len());
            let name = format!("temper-{}", &class[..end]);
            assert!(!name.starts_with("temper-reserved-"), "stylesheet selects reserved slot: {name}");
            assert!(NAMES.styles.contains(&name.as_bytes()), "stylesheet selects unknown look: {name}");
            classes.insert(name);
        }
    }
    assert!(!classes.is_empty(), "stylesheet selects look classes");
}
#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary tests read the browser stylesheet as text")]
fn the_switchers_phone_links_hide_only_on_wide_pages() {
    assert_eq!(NAMES.style(PROJECTS_PHONE), Some(b"temper-projects-phone".as_slice()));
    let css = include_str!("../../../temper-client-shell-web/temper.css");
    assert!(css.contains(
        "@media (min-width: 761px) {\n  #app > .temper-rail .temper-projects-acts > .temper-projects-phone { display: none; }\n}"
    ));
}

#[test]
fn model_line_styles_name_the_reason_and_acts() {
    for (style, name) in [
        (WELCOME_STATUS_WHY, b"temper-welcome-status-why".as_slice()),
        (WELCOME_STATUS_ACTS, b"temper-welcome-status-acts"),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn account_rows_and_sign_ins_have_distinct_style_names() {
    assert_eq!(NAMES.style(SETTINGS_ROW), Some(b"temper-settings-row".as_slice()));
    assert_eq!(NAMES.style(MODEL_SIGN_IN), Some(b"temper-model-sign-in".as_slice()));
    assert_ne!(SETTINGS_ROW, MODEL_SIGN_IN);
}

#[test]
fn the_settings_controls_have_their_own_names_and_small_element() {
    for (style, name) in [
        (SETTINGS_CHECKS, b"temper-settings-checks".as_slice()),
        (SETTINGS_SWITCH, b"temper-settings-switch".as_slice()),
        (SETTINGS_TOGGLE_ROW, b"temper-settings-toggle-row".as_slice()),
        (SETTINGS_TEXT, b"temper-settings-text".as_slice()),
        (SETTINGS_FACTS, b"temper-settings-facts".as_slice()),
        (SETTINGS_REFUSAL, b"temper-settings-refusal".as_slice()),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
    assert!(ELEMENTS.contains(&(owner(), Role::Text, SETTINGS_DETAIL, "small")));
}

#[test]
fn asking_opening_and_needs_have_their_css_names() {
    for (style, name) in [
        (STARTER_LEDE, b"temper-starter-lede".as_slice()),
        (STARTER_WAYS, b"temper-starter-ways"),
        (WAYS_LEDE, b"temper-ways-lede"),
        (WAYS, b"temper-ways"),
        (WAYS_TABS, b"temper-ways-tabs"),
        (WAYS_NOW, b"temper-ways-now"),
        (WAYS_AXIS, b"temper-ways-axis"),
        (WAYS_PANEL, b"temper-ways-panel"),
        (WAYS_SAYS, b"temper-ways-says"),
        (WAYS_ZOOM, b"temper-ways-zoom"),
        (WAYS_ZOOM_HEAD, b"temper-ways-zoom-head"),
        (WAYS_FOOT, b"temper-ways-foot"),
        (WAYS_LEGEND, b"temper-ways-legend"),
        (WAYS_KIND_AGENT, b"temper-ways-kind-agent"),
        (WAYS_KIND_PERSON, b"temper-ways-kind-person"),
        (WAYS_KIND_CODE, b"temper-ways-kind-code"),
        (WAYS_KIND_LINE, b"temper-ways-kind-line"),
        (WAYS_ACTS, b"temper-ways-acts"),
        (WAYS_USE, b"temper-ways-use"),
        (WAYS_REFUSED, b"temper-ways-refused"),
        (TOAST, b"temper-toast"),
        (NEEDS, b"temper-needs"),
        (WHY, b"temper-why"),
        (NEEDS_BAR, b"temper-needs-bar"),
        (NEEDS_BAR_AT, b"temper-needs-bar-at"),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn path_styles_keep_their_css_names() {
    for (token, name) in [
        (WAYS_MAYBE, b"temper-ways-maybe".as_slice()),
        (WAYS_WHY, b"temper-ways-why"),
        (WAYS_RISE_PERSON, b"temper-ways-rise-person"),
        (WAYS_RISE_AGENT, b"temper-ways-rise-agent"),
        (WAYS_FADE_PERSON, b"temper-ways-fade-person"),
        (WAYS_FADE_PERSON_FAINT, b"temper-ways-fade-person-faint"),
        (WAYS_FADE_AGENT, b"temper-ways-fade-agent"),
        (WAYS_FADE_AGENT_FAINT, b"temper-ways-fade-agent-faint"),
        (WAYS_FADE_BASE, b"temper-ways-fade-base"),
        (WAYS_KIND_MAYBE, b"temper-ways-kind-maybe"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
fn settings_styles_keep_their_css_names_beside_main_pages() {
    for (style, name) in [
        (SETTINGS_COL, b"temper-settings-col".as_slice()),
        (SETTINGS_INTRO, b"temper-settings-intro".as_slice()),
        (SETTINGS_KICKER, b"temper-settings-kicker".as_slice()),
        (SETTINGS_HEADLINE, b"temper-settings-headline".as_slice()),
        (SETTINGS_LEDE, b"temper-settings-lede".as_slice()),
        (SETTINGS_TABS, b"temper-settings-tabs".as_slice()),
        (SETTINGS_BODY, b"temper-settings-body".as_slice()),
        (SETTINGS_SETTING, b"temper-settings-setting".as_slice()),
        (SETTINGS_SIGN_OUT, b"temper-settings-sign-out"),
        (SETTINGS_SECTION, b"temper-settings-section".as_slice()),
        (SETTINGS_SECTION_HEAD, b"temper-settings-section-head".as_slice()),
        (SETTINGS_SECTION_TITLE, b"temper-settings-section-title".as_slice()),
        (SETTINGS_LINES, b"temper-settings-lines".as_slice()),
        (SETTINGS_WHEN, b"temper-settings-when".as_slice()),
        (SETTINGS_BAND, b"temper-settings-band".as_slice()),
        (SETTINGS_QUIET_BAND, b"temper-settings-quiet-band".as_slice()),
        (SETTINGS_TAG, b"temper-settings-tag".as_slice()),
        (SETTINGS_BTN, b"temper-settings-btn".as_slice()),
        (SETTINGS_PRIMARY, b"temper-settings-primary".as_slice()),
        (SETTINGS_SMALL, b"temper-settings-small".as_slice()),
        (SETTINGS_DETAIL, b"temper-settings-detail".as_slice()),
        (SETTINGS_END, b"temper-settings-end".as_slice()),
        (SETTINGS_ROW, b"temper-settings-row".as_slice()),
        (SETTINGS_PATH, b"temper-settings-path".as_slice()),
        (SETTINGS_TEXT, b"temper-settings-text".as_slice()),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn welcome_and_repository_styles_keep_their_css_names() {
    for (style, name) in [
        (WELCOME_FORGE_STATUS, b"temper-welcome-forge-status".as_slice()),
        (WELCOME_SPIN, b"temper-welcome-spin"),
        (WELCOME_ACTS, b"temper-welcome-acts"),
        (WELCOME_OK, b"temper-welcome-ok"),
        (WELCOME_SEP, b"temper-welcome-sep"),
        (WELCOME_PANEL, b"temper-welcome-panel"),
        (WELCOME_AWAY, b"temper-welcome-away"),
        (WELCOME_AGAIN, b"temper-welcome-again"),
        (WELCOME_PASTE, b"temper-welcome-paste"),
        (WELCOME_FIELD, b"temper-welcome-field"),
        (WELCOME_CMD, b"temper-welcome-cmd"),
        (HOME_COPY, b"temper-home-copy"),
        (HOME_SMALL, b"temper-home-small"),
    ] {
        assert_eq!(NAMES.style(style), Some(name));
    }
}

#[test]
fn decision_tokens_name_the_away_rows() {
    for (token, name) in [
        (DECIDED, b"temper-decided".as_slice()),
        (DECISION, b"temper-decision"),
        (DECISION_ALLOWED, b"temper-decision-allowed".as_slice()),
        (DECISION_TOOK, b"temper-decision-took"),
        (DECISION_WHY, b"temper-decision-why"),
        (DECISION_LATE, b"temper-decision-late"),
        (DECISION_OVERTURNED, b"temper-decision-overturned"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
fn work_tokens_name_the_line_and_its_rows() {
    for (token, name) in [
        (WORK_RUN, b"temper-work-run".as_slice()),
        (WORK_RUNNING, b"temper-work-running"),
        (WORK_FAILED, b"temper-work-failed"),
        (WORK_CALLS, b"temper-work-calls"),
        (WORK_OUT, b"temper-work-out"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
fn the_writing_end_and_reading_controls_have_their_css_names() {
    for (token, name) in [
        (WRITING_END, b"temper-writing-end".as_slice()),
        (LATEST, b"temper-latest"),
        (THREAD_LOADING, b"temper-thread-loading"),
    ] {
        assert_eq!(NAMES.style(token), Some(name));
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests parse the phone stylesheet's selectors and pixel heights"
)]
fn each_phone_target_stands_44_pixels() {
    let mut source = include_str!("../../../temper-client-shell-web/temper.css");
    let mut rules = Vec::new();
    while let Some((_, after)) = source.split_once("@media (max-width: 760px)") {
        let (_, body) = after.split_once('{').expect("phone media body");
        let mut depth = 1_u32;
        let end = body
            .char_indices()
            .find_map(|(at, ch)| {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                (depth == 0).then_some(at)
            })
            .expect("phone media ends");
        for rule in body[..end].split('}') {
            if let Some((selectors, declarations)) = rule.split_once('{') {
                // A section comment is not part of its first selector.
                let selectors = selectors.rsplit("*/").next().unwrap();
                rules.push((selectors.trim(), declarations));
            }
        }
        source = &body[end + 1..];
    }
    for target in [
        "#app > .temper-rail .temper-projects > summary",
        "#app > .temper-rail > .temper-away-chip::after",
        "#app > .temper-rail > .temper-night-switch::after",
        "#app > .temper-tabbar > a",
        "#app > .temper-session .temper-session-composer > button::after",
        "#app > .temper-session .temper-model > summary::before",
        "#app > .temper-session .temper-model-menu button::before",
        "#app > .temper-session .temper-model-menu a::before",
        "#app > .temper-session .temper-model-find",
        "#app > .temper-session .temper-ways-tabs > [role=tab]::after",
        "#app > .temper-session .temper-ways-acts > button::after",
        "#app > .temper-session .temper-ways-zoom-head > a::after",
        "#app > .temper-session .temper-crumbs > .temper-back::after",
        "#app > .temper-session .temper-session-right > button::after",
        "#app > .temper-home .temper-need-acts .temper-recommended",
        "#app > .temper-home .temper-need-card > .temper-need-more",
        "#app > .temper-home .temper-home-acts > :is(a, button)::after",
        "#app > .temper-home .temper-home-small::after",
        "#app > .temper-home .temper-home-skip::after",
        "#app > .temper-home .temper-home-head > :is(a, button)::after",
        "#app > .temper-home > .temper-home-phone > a",
        "#app > .temper-home .temper-home-rows > li > a",
        "#app > .temper-needs .temper-ask-choices button",
        "#app > .temper-needs .temper-ask-choices a.temper-recommended",
        "#app > .temper-needs .temper-ask-choices > details > summary",
        "#app > .temper-needs .temper-need-tell > button::after",
        "#app > .temper-needs .temper-ask-quiet::after",
        "#app > .temper-needs .temper-ask-saw > summary::after",
        "#app > .temper-needs .temper-ask-context > a::after",
        "#app > .temper-needs .temper-needs-home::after",
        "#app > .temper-needs .temper-decision > button::after",
        "#app > :is(.temper-home, .temper-needs) .temper-answered-undo::after",
        "#app > .temper-needs .temper-needs-bar > a::after",
        "#app > .temper-work-page .temper-work-chip::after",
        "#app > .temper-work-page .temper-work-card a::after",
        "#app > .temper-work-page .temper-work-row > a::after",
        "#app > .temper-work-page .temper-work-section-head > a::after",
    ] {
        assert!(
            rules.iter().any(|(selectors, declarations)| {
                selectors.split(",\n").any(|selector| selector.trim() == target)
                    && declarations.split(';').any(|declaration| {
                        let Some((property, value)) = declaration.split_once(':') else { return false };
                        property.trim() == "min-height"
                            && value
                                .trim()
                                .strip_suffix("px")
                                .and_then(|n| n.parse::<u32>().ok())
                                .is_some_and(|height| height >= 44)
                    })
            }),
            "phone target needs at least 44 px: {target}"
        );
    }
}

#[test]
fn team_and_how_resolve_to_distinct_style_names() {
    assert_eq!(NAMES.style(TEAM), Some(b"temper-team".as_slice()));
    assert_eq!(NAMES.style(HOW_WIDE), Some(b"temper-how-wide".as_slice()));
}
