use super::*;
pub fn family_request(record: Box<[u8]>) -> Box<[u8]> {
    let value =
        w::FamilyRequest { scope: w::Scope::Own, record };
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured family request");
    writer.finish()
}
pub fn family_record(record: Box<[u8]>) -> Box<[u8]> {
    let window = w::Window { rows: 3, after: None };
    let value = w::FamilyRecord { scope: w::Scope::Own, window, record };
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured family record");
    writer.finish()
}

pub fn not_served(owner: w::Owner) -> w::NotServed {
    let lim = wire();
    let family = w::Owner::Family(w::Family { number: 9 });
    let mut served = List::with_capacity(2);
    for (owner, first, last) in
        [(w::Owner::Jig, jig_client_protocol_wire::VERSION, jig_client_protocol_wire::VERSION), (family, 2, 2)]
    {
        served.push(w::Served { owner, first, last }).expect("range slot");
    }
    w::NotServed { owner, served }
}
