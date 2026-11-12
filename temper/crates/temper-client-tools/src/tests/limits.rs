use crate::limits;
#[test]
fn each_ceiling_is_within_the_client_limit_it_reads_from() {
    let client = limits::client().client.domain;
    // The reference delegation needs eighteen executor places.
    assert_eq!(client.authority, 18);
    assert_eq!(client.delegates, 80);
    assert_eq!(limits::ROWS, client.rows, "a page's rows are the client's rows per window");
    assert_eq!(limits::PART, 65_536, "a page holds 64 KiB");
    assert_eq!(client.text, limits::CLIENT_TEXT, "the client's text keeps its own bound");
    assert_eq!(client.turn_text, limits::CLIENT_TEXT, "the client's turn text keeps its own bound");
    assert!(client.text <= limits::TEXT, "the client's text fits a result's text");
    assert_eq!(limits::tools().words, client.text, "an act's words use the client's text bound");
    assert_eq!(
        crate::jig::form_limits(&limits::tools()).text,
        limits::CLIENT_TEXT,
        "the settings form keeps the client's text"
    );
    assert!(u64::from(limits::TEXT) <= client.document_bytes, "a result's text fits one document");
    assert!(limits::PART <= client.turn_bytes, "a page fits a turn");
    assert_eq!(
        crate::jig::render::task_source::capacity(&limits::tools()),
        usize::try_from(limits::PART).expect("page fits usize"),
        "a task text page holds a whole page"
    );
    assert!(limits::WAIT <= limits::WAIT_MOST && limits::LOCK_TRY <= limits::LOCK, "each default within its most");
}
#[test]
fn the_tools_preset_covers_deployments_with_its_windows_and_retries() {
    let mut terminal = temper_client_shell_terminal::limits::native().client;
    terminal.client.domain.rows = limits::ROWS;
    terminal.client.domain.text = limits::CLIENT_TEXT;
    terminal.client.domain.turn_text = limits::CLIENT_TEXT;
    terminal.client.domain.requests.most = skein_lib::Duration::from_secs(2);
    terminal.client.domain.backoff.most = skein_lib::Duration::from_secs(2);
    assert_eq!(limits::client(), terminal);
}
#[test]
fn the_rows_repeats_are_within_the_client_limits() {
    use crate::jig::table::Occurs;
    let limits = limits::client();
    for row in crate::table::TOOLS {
        let most = if row.name == crate::jig::rows::CHANGE_SETTINGS.name {
            crate::jig::settings_file::FILES
        } else if row.name == crate::adopt::ROW.name {
            limits.forge.checks
        } else if row.name == crate::jig::rows::AMEND.name {
            limits.client.domain.rows
        } else if row.name == crate::jig::rows::CHAT_CREATE.name || row.name == crate::jig::rows::GOAL_START.name {
            jig_domain_tasks::MODELS
        } else {
            limits.client.domain.delegates
        };
        for form in row.forms {
            for argument in *form {
                match argument.occurs {
                    Occurs::Repeated { most: repeated } => assert!(repeated <= most, "{:?}", row.name),
                    Occurs::Required | Occurs::Optional => {}
                }
            }
        }
    }
}

#[test]
fn the_help_fits_the_tools_window() {
    let window = usize::try_from(limits::TEXT).expect("text fits usize");
    let mut measured = skein_lib::Writer::new(window);
    crate::help::text(None, &mut measured).expect("the help fits its text window");
    assert!(measured.written() <= window);
}

#[test]
fn retries_stay_short_beside_the_deadline() {
    let client = limits::client().client.domain;
    let first = skein_lib::Duration::from_secs(1);
    let most = skein_lib::Duration::from_secs(2);
    assert_eq!(client.requests.first, first);
    assert_eq!(client.backoff.first, first);
    assert_eq!(client.requests.most, most);
    assert_eq!(client.backoff.most, most);
    assert!(most.as_nanos() < limits::DEADLINE.as_nanos());
    assert_eq!(limits::DEADLINE, skein_lib::Duration::from_secs(30));
}
