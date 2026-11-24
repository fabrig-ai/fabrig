use hearth_conformance_world::{
    fake_mmdebstrap::{self, Spawned},
    resolve,
};

#[test]
fn the_fake_prints_each_mmdebstrap_recording_as_recorded() {
    let (mirror, index) = fake_mmdebstrap::committed();
    let mut mirror = Some(mirror);
    let mut stanzas_compared = 0;
    for (name, real) in hearth_recordings::all("mmdebstrap") {
        stanzas_compared += usize::from(real.stanzas.is_some());
        let case = resolve::case(&name);
        let fake =
            fake_mmdebstrap::run(&mut mirror, case, &Spawned { argv: &real.argv, env: &real.env, dir: &real.dir });
        let cut;
        let stanzas = if let Some(stanzas) = &real.stanzas {
            stanzas
        } else {
            let insts = fake_mmdebstrap::insts(&real);
            cut =
                resolve::stanzas(&index, &insts.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap_or_else(|line| {
                    panic!("{} {name}: no committed stanza for {}", real.version, String::from_utf8_lossy(&line))
                });
            &cut
        };
        assert_eq!(fake_mmdebstrap::differ(case, &real, &fake, Some(stanzas)), None, "{} {name}", real.version);
    }
    assert!(stanzas_compared >= 1, "test-min’s recorded stanzas were compared");
}
