use std::{env, fs, path::Path, process};
use temper_client_journeys::{JOURNEYS, Refusal, add, reprint};
use temper_client_shell_web::journey;

fn main() {
    skein_fail_stop::install();
    let mut args = env::args_os();
    let _program = args.next();
    let arguments: Vec<_> = args.collect();
    let result = match arguments.as_slice() {
        [] => reprint_all(),
        [command, name, file] if command == "add" => {
            let Some(name) = name.to_str() else {
                fail(&format!("invalid journey name {}", name.display()));
            };
            match fs::read(file) {
                Ok(recording) => add(Path::new(JOURNEYS), name, &recording).map(|()| false).map_err(refusal),
                Err(_) => Err(format!("cannot read {}", Path::new(file).display())),
            }
        }
        _ => {
            eprintln!("usage: temper-client-journeys [add NAME FILE]");
            process::exit(2);
        }
    };
    match result {
        Ok(false) => {}
        Ok(true) => process::exit(1),
        Err(why) => fail(&why),
    }
}

fn reprint_all() -> Result<bool, String> {
    let stale = reprint(Path::new(JOURNEYS), Path::new(JOURNEYS)).map_err(refusal)?;
    let mut names = Vec::new();
    for entry in fs::read_dir(JOURNEYS).map_err(|_| format!("cannot read {JOURNEYS}"))? {
        let entry = entry.map_err(|_| format!("cannot read {JOURNEYS}"))?;
        let path = entry.path();
        if entry.file_type().map_err(|_| format!("cannot read {}", path.display()))?.is_dir()
            && path.join("recording").is_file()
        {
            names.push(entry.file_name());
        }
    }
    names.sort();
    for name in names {
        println!("{}", name.to_string_lossy());
    }
    for (name, stopped) in &stale {
        eprintln!("temper-client-journeys: {}", stale_message(name, stopped));
    }
    Ok(!stale.is_empty())
}

fn refusal(why: Refusal) -> String {
    match why {
        Refusal::Name(name) => format!("invalid journey name {name:?}; expected [a-z][a-z0-9-]*"),
        Refusal::Proof { name, refusal } => match refusal {
            journey::Refusal::Unreadable(unreadable) => unreadable_message(&name, &unreadable),
            journey::Refusal::Stale(stale) => stale_message(&name, &stale),
            journey::Refusal::Unproved { mark } => format!("{name} has no live print at mark {mark}"),
            journey::Refusal::Differs { mark, byte } => format!("{name} differs at mark {mark}, byte {byte}"),
        },
        Refusal::Unreadable { name, unreadable } => unreadable_message(&name, &unreadable),
        Refusal::Io { path } => format!("cannot read or write {}", path.display()),
    }
}

fn unreadable_message(name: &str, unreadable: &journey::Unreadable) -> String {
    format!("{name} is unreadable at line {}: {}", unreadable.line, unreadable.why)
}

fn stale_message(name: &str, stale: &journey::Stale) -> String {
    format!(
        "{name} is stale at turn {}, line {}: recorded {}, asked {}; record it again (client/journeys.md, 8)",
        stale.turn, stale.line, stale.recorded, stale.asked
    )
}

fn fail(why: &str) -> ! {
    eprintln!("temper-client-journeys: {why}");
    process::exit(1);
}
