//! Every representable wall day compared with Gregorian rollover (testing-strategy.md, 8).

use skein_lib::Wall;
use skein_ui::Owner;
use skein_ui::phrases::{
    Form, Hole, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseLimits, PhraseShape, Phrasebook, PluralClass,
    PluralRule, Segment, SpanUnit, WallFormat,
};

const ID: PhraseId = PhraseId { owner: Owner { number: 4 }, number: 1 };
const LIMITS: PhraseLimits =
    PhraseLimits { languages: 2, phrases: 2, holes: 6, segments: 60, text_bytes: 200, hole_bytes: 8 };
fn wall_shape(kind: HoleKind) -> Box<[PhraseShape]> {
    Box::new([PhraseShape { id: ID, holes: Box::new([HoleShape { kind, bytes: 0 }]), plural: None }])
}
fn wall_language(kind: HoleKind, wall: WallFormat) -> Language {
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Seconds,
        wall,
        phrases: Box::new([Phrase {
            id: ID,
            forms: Box::new([Form {
                class: PluralClass::Other,
                segments: Box::new([Segment::Hole { index: 0, kind }]),
            }]),
        }]),
    }
}
fn wall_book(kind: HoleKind, format: WallFormat) -> Phrasebook {
    Phrasebook::load(wall_shape(kind), Box::new([wall_language(kind, format)]), &LIMITS)
        .expect("the date phrase fits its shape and limits")
}

#[test]
fn every_representable_wall_day_matches_gregorian_rollover() {
    let book = wall_book(HoleKind::Date, WallFormat::Nanos);
    let months: &[(&[u8], u64)] = &[
        (b"Jan", 31),
        (b"Feb", 28),
        (b"Mar", 31),
        (b"Apr", 30),
        (b"May", 31),
        (b"Jun", 30),
        (b"Jul", 31),
        (b"Aug", 31),
        (b"Sep", 30),
        (b"Oct", 31),
        (b"Nov", 30),
        (b"Dec", 31),
    ];
    let (mut year, mut month, mut day) = (1970_u64, 0, 1);
    let last = Wall::from_nanos(u64::MAX).as_secs() / 86_400;
    for elapsed in 0..=last {
        let seconds = elapsed * 86_400;
        let wall = Wall::from_nanos(seconds * 1_000_000_000);
        let (name, days) = months[month];
        let decimal = skein_lib::Decimal::of(day);
        let mut expected = skein_lib::Writer::new(name.len() + 1 + decimal.as_bytes().len());
        expected.put(name).unwrap();
        expected.put(b" ").unwrap();
        expected.put(decimal.as_bytes()).unwrap();
        assert_eq!(
            book.say(0, ID, &[Hole::Wall(wall)]).unwrap().text.as_ref(),
            expected.finish().as_ref(),
            "{year}-{month}-{day}"
        );
        let leap = month == 1 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        if day == days + u64::from(leap) {
            day = 1;
            month += 1;
            if month == months.len() {
                month = 0;
                year += 1;
            }
        } else {
            day += 1;
        }
    }
}
