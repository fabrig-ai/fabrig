pub(crate) mod phrases;

#[test]
fn an_untooled_wait_names_the_worker_without_blaming_its_login() {
    use jig_client::contract as c;
    let words = crate::words::Words::new(crate::words::phrases::english::english(128).unwrap(), 0, 0);
    let limits =
        skein_ui::Limits { nodes: 3, depth: 2, text_per_node: 256, text_bytes: 256, ..skein_ui::Limits::EMPTY };
    let mut builder = skein_ui::Builder::new(&limits);
    builder.open(skein_ui::Role::Paragraph);
    words.run_wait(
        &mut builder,
        &c::TaskRunWait::Account { account: Some(4), source: Some(7), why: c::TaskWaitWhy::Untooled { worker: 2 } },
    );
    builder.close();
    let tree = builder.finish();
    let mut said = skein_lib::List::with_capacity(128);
    for node in tree.nodes() {
        for run in &node.text {
            for byte in &run.text {
                said.push(*byte).unwrap();
            }
        }
    }
    assert_eq!(said.as_slice(), b"The program started without its tools on worker 2.");
}

#[test]
fn a_linked_worker_fault_tells_the_person_to_stop_it_first() {
    use jig_client::contract as c;
    let words = crate::words::Words::new(crate::words::phrases::english::english(128).unwrap(), 0, 0);
    let limits =
        skein_ui::Limits { nodes: 3, depth: 2, text_per_node: 256, text_bytes: 256, ..skein_ui::Limits::EMPTY };
    let mut builder = skein_ui::Builder::new(&limits);
    builder.open(skein_ui::Role::Paragraph);
    words.refusal(
        &mut builder,
        &c::Refusal::Malformed {
            at: 0,
            why: c::Malformed::Settings(c::MalformedSettings {
                setting: c::Setting::Worker(c::SettingNumber { number: 2 }),
                fault: c::SettingFault::Linked,
                detail: None,
            }),
        },
        128,
    );
    builder.close();
    let tree = builder.finish();
    let mut said = skein_lib::List::with_capacity(64);
    for node in tree.nodes() {
        for run in &node.text {
            for byte in &run.text {
                said.push(*byte).unwrap();
            }
        }
    }
    assert_eq!(said.as_slice(), b"Worker 2 is linked: stop it first");
}
