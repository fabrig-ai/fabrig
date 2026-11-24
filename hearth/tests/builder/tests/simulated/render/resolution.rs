use hearth::Form;
use hearth_builder_world::{
    fixtures::{self, Laid},
    protocol,
};

fn render(form: Form, laid: &[Laid]) -> protocol::Run {
    let files = fixtures::files_of(&fixtures::resolution_golden().tree);
    let scenario =
        fixtures::Scenario { form, laid, ..fixtures::render_targets(&files, &[fixtures::TEST_MIN_TARGET], false) };
    protocol::run(&scenario, fixtures::SIMULATION_SEED)
}

#[test]
fn a_recorded_resolution_succeeds_and_prints_its_layer_keys() {
    let laid = [Laid::Resolution(fixtures::test_min_recorded())];
    let out = render(Form::Text, &laid);
    assert_eq!(
        fixtures::lines_starting(&out.stdout, &["packages ", "keys "]),
        "packages     part 78ec4b1fb53a   resolution 21e9a02feb65 known\nkeys         packages d018a0eea488   software f60610453970   config 8c9bf06f6262\n"
    );
    assert_eq!(out.code, Some(0));
}

#[test]
fn an_unrecorded_resolution_succeeds_and_prints_needed_without_keys() {
    let out = render(Form::Text, &[]);
    assert_eq!(
        fixtures::lines_starting(&out.stdout, &["packages ", "keys "]),
        "packages     part 78ec4b1fb53a   resolution 21e9a02feb65 needed\n"
    );
    assert_eq!(out.code, Some(0));
}

#[test]
fn a_needed_and_a_known_render_give_the_same_spec_and_packages_digests() {
    let needed = render(Form::JsonLines, &[]);
    let laid = [Laid::Resolution(fixtures::test_min_recorded())];
    let known = render(Form::JsonLines, &laid);
    assert_eq!(fixtures::spec_packages_digests(&known.stdout), fixtures::spec_packages_digests(&needed.stdout));
}

#[test]
fn the_json_spec_gives_the_resolution_and_its_keys_whole() {
    let laid = [Laid::Resolution(fixtures::test_min_recorded())];
    let out = render(Form::JsonLines, &laid);
    let text = fixtures::text(&out);
    assert!(text.contains("\"resolution\":{\"key\":\"21e9a02feb654883ef051d247cb14db878e04c03c2f7675c784a679ea0956d5d\",\"keys\":{\"config\":\"8c9bf06f62629795e2ee99885a9faf9ad94bd7a066f5a40f39739824ecb3bde6\",\"packages\":\"d018a0eea48807bb1966946d32e001cc2a2a6378bb257e8786e4961427de36a1\",\"software\":\"f6061045397016408cb4dcc2c3145f790f1ff970564964c7f9cbfd5c4c206b5c\"},\"kind\":\"known\"}"), "{text}");
}

#[test]
fn the_test_repositorys_targets_render_the_pins_resolution_inputs() {
    let out = render(Form::JsonLines, &[]);
    let text = fixtures::text(&out);
    assert!(text.contains("\"sources\":{\"architecture\":\"amd64\",\"archive\":\"20261017T143929Z\",\"components\":[\"main\"],\"suite\":\"forky\",\"updates\":false}"), "{text}");
}
