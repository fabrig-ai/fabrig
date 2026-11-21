use skein_fake_machine::Item;
use skein_kv::{Event, Range, Request};
use skein_kv_tests::golden::{KEPT, LAST, LIMITS, model, open};
use skein_lib::Token;
use std::collections::BTreeMap;

#[test]
fn the_kept_files_recover_to_the_models_map() {
    let items: Vec<Item> = KEPT.iter().map(|(name, bytes)| Item::file(name, bytes)).collect();
    let mut world = open(&items, LAST);
    let mut map = BTreeMap::new();
    let mut range = Range::prefix(b"");
    loop {
        world.request(Request::Load { owner: Token::new(4), range: range.clone(), max: LIMITS.page });
        let Event::Loaded { rows, next, .. } = world.pop() else {
            panic!("setup expected a loaded page");
        };
        for row in rows {
            map.insert(row.key, row.value);
        }
        match next {
            Some(start) => range.start = start,
            None => break,
        }
    }
    assert_eq!(map, model());
}
