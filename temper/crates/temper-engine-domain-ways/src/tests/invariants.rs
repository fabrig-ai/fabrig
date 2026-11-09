//! Path bounds (temper/docs/design/domain/forge.md, 8.13).
use super::*;
pub(crate) fn bounds(path: &Path) {
    assert!(path.stops.len() <= STOPS, "stop bound");
    assert_eq!(path.stops.last().expect("bar").actor, Actor::Bar, "final bar");
    for stop in &path.stops {
        assert!(stop.words.len() <= WORDS, "words bound");
        assert!(stop.over.len() <= OVER, "over bound");
        assert!(stop.card.who.len() <= WHO, "who bound");
        assert!(stop.card.name.len() <= NAME, "name bound");
        assert!(stop.card.facts.len() <= FACTS, "facts bound");
        for text in [&stop.words, &stop.over, &stop.card.who, &stop.card.name] {
            assert!(skein_lib::bytes::is_utf8(text), "text ends at a character boundary");
        }
        for fact in &stop.card.facts {
            assert!(fact.label.len() <= LABEL, "label bound");
            let bound = if fact.label.as_ref() == b"Asked to" { ASKED } else { VALUE };
            assert!(fact.value.len() <= bound, "value bound");
            assert!(skein_lib::bytes::is_utf8(&fact.value), "value ends at a character boundary");
        }
    }
}
