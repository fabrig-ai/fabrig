use crate::{ForgeStart, route::forge_route::workspace_repository};
use alloc::boxed::Box;
use temper_engine_domain_forge::{self as forge, items};

fn item(start: items::Start) -> items::Item {
    items::Item {
        provider: skein_forge::api::Repository { forge: 1, repository: 7 },
        name: b"repo".as_slice().into(),
        host: b"forge.example".as_slice().into(),
        owner: b"acme".as_slice().into(),
        start,
        push: Some(b"temper/7".as_slice().into()),
        landing: Some(b"main".as_slice().into()),
        holder: Some(7),
        credential: Some(1),
        bytes: 0,
    }
}

#[test]
fn a_merged_item_maps_to_a_merged_forge_start_saving_as_a_base() {
    let heads = Box::new([[4; 32], [3; 32], [4; 32]]);
    for push in [Some(Box::from(b"temper/7".as_slice())), None] {
        let base = item(items::Start::Base(b"main".as_slice().into()));
        let merged = item(items::Start::Merged { branch: b"main".as_slice().into(), heads: heads.clone() });
        let merged = workspace_repository(merged, push.clone(), b"temper/s7".as_slice().into()).unwrap();
        let mut expected = workspace_repository(base, push.clone(), b"temper/s7".as_slice().into()).unwrap();
        assert_eq!(expected.save.as_deref(), if push.is_some() { Some(b"temper/s7".as_slice()) } else { None });
        assert_eq!(merged.push, push);
        assert_eq!(merged.landing.as_deref(), if push.is_some() { Some(b"main".as_slice()) } else { None });
        expected.start = ForgeStart::Merged { branch: b"main".as_slice().into(), heads: heads.clone() };
        assert_eq!(merged, expected);
        assert_eq!(merged.tag, forge::tag(1, 7));
    }
}
