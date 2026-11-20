//! An address's text without formatting (runtime/lib.md, 6).
use crate::{Decimal, Writer};
use alloc::boxed::Box;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Writes an address byte for byte as core's `IpAddr` display does.
/// IPv4-mapped IPv6 addresses stay mapped; callers canonicalize where needed.
#[must_use]
pub fn ip_text(ip: IpAddr) -> Box<[u8]> {
    let mut measure = Writer::measure(45);
    put(ip, &mut measure);
    let mut out = Writer::new(measure.written());
    put(ip, &mut out);
    out.finish()
}

fn put(ip: IpAddr, out: &mut Writer) {
    match ip {
        IpAddr::V4(ip) => v4(ip, out),
        IpAddr::V6(ip) => v6(ip, out),
    }
}
fn v4(ip: Ipv4Addr, out: &mut Writer) {
    for (at, byte) in ip.octets().iter().enumerate() {
        if at != 0 {
            out.put(b".").expect("measured address");
        }
        out.put(Decimal::of(u64::from(*byte)).as_bytes()).expect("measured address");
    }
}
fn v6(ip: Ipv6Addr, out: &mut Writer) {
    if let Some(mapped) = ip.to_ipv4_mapped() {
        out.put(b"::ffff:").expect("measured address");
        v4(mapped, out);
        return;
    }
    let groups = ip.segments();
    let mut start = 0;
    let mut longest = 0;
    let mut run = 0_usize;
    for (at, group) in groups.iter().enumerate() {
        if *group == 0 {
            run = run.checked_add(1).expect("eight groups");
            if run > longest {
                longest = run;
                start = at.checked_add(1).expect("eight groups").checked_sub(run).expect("run before its end");
            }
        } else {
            run = 0;
        }
    }
    let end = start.checked_add(longest).expect("eight groups");
    for (at, group) in groups.iter().enumerate() {
        if longest >= 2 && at >= start && at < end {
            if at == start {
                out.put(b"::").expect("measured address");
            }
            continue;
        }
        if at != 0 && !(longest >= 2 && at == end) {
            out.put(b":").expect("measured address");
        }
        let mut leading = true;
        for shift in [12_u32, 8, 4, 0] {
            let nibble = u8::try_from((group >> shift) & 0xf).expect("four bits");
            if nibble != 0 || !leading || shift == 0 {
                out.put(&[super::hex_digit(nibble)]).expect("measured address");
                leading = false;
            }
        }
    }
}
