//! The parser against the engine's renderer, not a second formatter (redeploy.md, 8).
use jig_protocol_worker_wire::Agreed;
use temper_redeploy_tool::openings::{Event, parse};

#[test]
fn rendered_worker_openings_parse_back_to_names_and_ordered_wires_seeded() {
    let mut random = 0x71e2_93af_u64;
    for seed in 0..2048 {
        let worker = if seed < 5 {
            ["", "worker with spaces", "λ worker", "a opened: b", "a refused at its opening: b"][seed]
                .as_bytes()
                .to_vec()
        } else {
            let length = usize::try_from(next(&mut random) % 64).unwrap();
            (0..length).map(|_| u8::try_from(next(&mut random) % 256).unwrap()).collect()
        };
        let names = ["jig-worker-wire", "smith-pieces", "smith-charter", "smith-transcript", "temper-worker-wire"];
        let length = u32::try_from(next(&mut random) % 9).unwrap();
        let mut wires = skein_lib::List::with_capacity(length);
        let mut expected = Vec::new();
        for _ in 0..length {
            let name = names[usize::try_from(next(&mut random) % 5).unwrap()];
            let version = u16::try_from(next(&mut random) % 65536).unwrap();
            wires.push(Agreed { name, version }).unwrap();
            expected.push((name.into(), version));
        }
        let name = String::from_utf8_lossy(&worker).into_owned();
        let line = temper_engine_shell::lines::note_line(
            "temper-engine",
            temper_engine_service::Note::WorkerOpened { worker: worker.into_boxed_slice(), wires },
        );
        assert_eq!(parse(&line), Some(Event::Opened { worker: name, wires: expected }), "seed {seed}: {line:?}");
    }
}
fn next(random: &mut u64) -> u64 {
    *random ^= *random << 13;
    *random ^= *random >> 7;
    *random ^= *random << 17;
    *random
}
