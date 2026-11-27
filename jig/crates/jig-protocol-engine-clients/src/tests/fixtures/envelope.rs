use super::*;
pub fn family_request(record: Box<[u8]>) -> Box<[u8]> {
    let value = w::FamilyRequest { scope: w::Scope::Own, record };
    let mut writer = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured family request");
    writer.finish()
}
pub fn family_record(record: Box<[u8]>) -> Box<[u8]> {
    family_record_at(record, None)
}

fn family_record_at(record: Box<[u8]>, after: Option<w::Cursor>) -> Box<[u8]> {
    let window = w::Window { rows: 3, after };
    let value = w::FamilyRecord { scope: w::Scope::Own, window, record };
    let mut writer = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured family record");
    writer.finish()
}

pub fn family_owned(record: Box<[u8]>, after: Option<w::Cursor>) -> w::Owned {
    let owner = w::Owner::Family(w::Family { number: 9 });
    w::Owned { owner, version: 2, record: family_record_at(record, after) }
}

pub const FAMILY_AFTER: core::documents::Cursor = core::documents::Cursor::Number { number: 7 };

pub fn family_window_cursor() -> w::Cursor {
    let value = crate::translate::encode_window(&FAMILY_AFTER, w::CEILINGS.cursor_value).expect("window cursor");
    w::Cursor { value }
}

pub fn foreign_window_cursor() -> w::Cursor {
    let mut cursor = family_window_cursor();
    *cursor.value.get_mut(size_of::<u16>()).expect("first layout byte") ^= 1;
    cursor
}

pub fn not_served(owner: w::Owner) -> w::NotServed {
    let family = w::Owner::Family(w::Family { number: 9 });
    let mut served = List::with_capacity(2);
    for (owner, first, last) in
        [(w::Owner::Jig, jig_client_protocol_wire::VERSION, jig_client_protocol_wire::VERSION), (family, 2, 2)]
    {
        served.push(w::Served { owner, first, last }).expect("range slot");
    }
    w::NotServed { owner, served }
}
