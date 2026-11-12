use std::path::PathBuf;
use std::process::ExitCode;

use skein_ui_bundle::{Inputs, Placeholder, Worker, bundle_with, glue};

const USAGE: &str = "usage: skein-ui-bundle --module PATH --name NAME --prefix /PREFIX/ --out DIR [--worker NAME=PATH] [--stylesheet PATH]… [--pictures OWNER-PREFIX=DIR]… [--page PATH] [--files DIR] [--noscript WORDS] [--gzip on|off] [--title WORDS] [--placeholder PATH --placeholder-stylesheet PATH]";

#[derive(Debug, PartialEq, Eq)]
struct Arguments {
    module: PathBuf,
    worker: Option<(String, PathBuf)>,
    out: PathBuf,
    inputs: Inputs,
}

fn arguments(args: &[String]) -> Result<Arguments, &'static str> {
    let mut module = None;
    let mut worker = None;
    let mut out = None;
    let mut name = None;
    let mut prefix = None;
    let mut inputs = Inputs {
        gzip: true,
        title: None,
        placeholder: None,
        name: String::new(),
        prefix: String::new(),
        stylesheets: Vec::new(),
        pictures: Vec::new(),
        page: None,
        files: None,
        noscript: "This application needs WebAssembly and scripts.".into(),
    };
    let mut noscript = None;
    let mut gzip = None;
    let mut markup = None;
    let mut stylesheet = None;
    let mut pairs = args.as_chunks::<2>().0;
    for pair in &mut pairs {
        let value = &pair[1];
        match pair[0].as_str() {
            "--module" if module.is_none() => module = Some(PathBuf::from(value)),
            "--worker" if worker.is_none() => {
                let (name, path) = value.split_once('=').ok_or(USAGE)?;
                if !segment(name) || path.is_empty() {
                    return Err(USAGE);
                }
                worker = Some((name.into(), path.into()));
            }
            "--out" if out.is_none() => out = Some(PathBuf::from(value)),
            "--name" if name.is_none() && segment(value) => name = Some(value.clone()),
            "--prefix" if prefix.is_none() && valid_prefix(value) => prefix = Some(value.clone()),
            "--stylesheet" => inputs.stylesheets.push(value.into()),
            "--pictures" => {
                let (owner, directory) = value.split_once('=').ok_or(USAGE)?;
                if !segment(owner) || directory.is_empty() {
                    return Err(USAGE);
                }
                inputs.pictures.push((owner.into(), directory.into()));
            }
            "--title" if inputs.title.is_none() => inputs.title = Some(value.clone()),
            "--placeholder" if markup.is_none() => markup = Some(PathBuf::from(value)),
            "--placeholder-stylesheet" if stylesheet.is_none() => stylesheet = Some(PathBuf::from(value)),
            "--page" if inputs.page.is_none() => inputs.page = Some(value.into()),
            "--files" if inputs.files.is_none() => inputs.files = Some(value.into()),
            "--noscript" if noscript.is_none() => noscript = Some(value.clone()),
            "--gzip" if gzip.is_none() => {
                gzip = Some(match value.as_str() {
                    "on" => true,
                    "off" => false,
                    _ => return Err(USAGE),
                });
            }
            _ => return Err(USAGE),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err(USAGE);
    }
    inputs.placeholder = match (markup, stylesheet) {
        (Some(markup), Some(stylesheet)) => Some(Placeholder { markup, stylesheet }),
        (None, None) => None,
        (Some(_), None) | (None, Some(_)) => return Err(USAGE),
    };
    inputs.gzip = gzip.unwrap_or(true);
    inputs.name = name.ok_or(USAGE)?;
    inputs.prefix = prefix.ok_or(USAGE)?;
    if let Some(words) = noscript {
        inputs.noscript = words;
    }
    Ok(Arguments { module: module.ok_or(USAGE)?, worker, out: out.ok_or(USAGE)?, inputs })
}

fn segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
}

fn valid_prefix(value: &str) -> bool {
    value
        .strip_prefix('/')
        .and_then(|value| value.strip_suffix('/'))
        .is_some_and(|middle| middle.split('/').all(segment))
}

#[expect(clippy::disallowed_methods, reason = "skein's round: reads the bundler command line")]
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = arguments(&args).map_err(str::to_owned)?;
    let page_glue = glue(&args.module, &args.inputs.name).map_err(|error| error.to_string())?;
    let worker = match args.worker {
        Some((name, path)) => {
            let glue = glue(&path, &name).map_err(|error| error.to_string())?;
            Some(Worker { name, glue })
        }
        None => None,
    };
    let digest =
        bundle_with(&args.inputs, &page_glue, worker.as_ref(), &args.out).map_err(|error| error.to_string())?;
    println!("{}", args.out.join(digest.to_string()).display());
    Ok(())
}

fn main() -> ExitCode {
    skein_fail_stop::install();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Arguments, Inputs, USAGE, arguments};

    const REQUIRED: &[&str] = &["--module", "notes.wasm", "--name", "notes", "--prefix", "/static/", "--out", "out"];
    const OPTIONAL: &[&str] = &[
        "--title",
        "Notes",
        "--placeholder",
        "loading.html",
        "--placeholder-stylesheet",
        "loading.css",
        "--gzip",
        "off",
        "--worker",
        "world=world.wasm",
        "--stylesheet",
        "a.css",
        "--stylesheet",
        "b.css",
        "--pictures",
        "notes=images",
        "--page",
        "own.html",
        "--files",
        "files",
        "--noscript",
        "Enable scripts.",
    ];
    #[test]
    fn the_arguments_make_inputs() {
        let inputs = Inputs {
            gzip: false,
            title: Some("Notes".into()),
            placeholder: Some(skein_ui_bundle::Placeholder {
                markup: "loading.html".into(),
                stylesheet: "loading.css".into(),
            }),
            name: "notes".into(),
            prefix: "/static/".into(),
            stylesheets: vec!["a.css".into(), "b.css".into()],
            pictures: vec![("notes".into(), "images".into())],
            page: Some("own.html".into()),
            files: Some("files".into()),
            noscript: "Enable scripts.".into(),
        };
        let complete: Vec<String> = REQUIRED.iter().chain(OPTIONAL).map(|word| (*word).into()).collect();
        assert_eq!(
            arguments(&complete),
            Ok(Arguments {
                module: "notes.wasm".into(),
                worker: Some(("world".into(), "world.wasm".into())),
                out: "out".into(),
                inputs,
            })
        );
        let required = REQUIRED.iter().map(|word| (*word).into()).collect::<Vec<String>>();
        assert_eq!(arguments(&required).expect("worker is optional").worker, None);
        assert!(arguments(&required).expect("default gzip").inputs.gzip);
        let on = REQUIRED.iter().copied().chain(["--gzip", "on"]).map(str::to_owned).collect::<Vec<_>>();
        assert!(arguments(&on).expect("explicit gzip on").inputs.gzip);
        for tail in [
            &["--placeholder", "loading.html"][..],
            &["--placeholder-stylesheet", "loading.css"],
            &["--gzip"][..],
            &["--gzip", "invalid"],
            &["--unknown", "x"][..],
            &["--name", "second"],
            &["--pictures", "images"],
            &["--pictures", "../=images"],
            &["--page"],
            &["--prefix", "/other/"],
            &["--worker", "world=one.wasm", "--worker", "other=two.wasm"],
            &["--worker", "world.wasm"],
            &["--worker"],
            &["--worker", "world="],
            &["--worker", "=world.wasm"],
            &["--worker", ".=world.wasm"],
            &["--worker", "..=world.wasm"],
            &["--worker", "../world=world.wasm"],
            &["--worker", "world/name=world.wasm"],
            &["--worker", "world name=world.wasm"],
            &["--worker", "world\"=world.wasm"],
        ] {
            let args = REQUIRED.iter().chain(tail).map(|word| (*word).into()).collect::<Vec<String>>();
            assert_eq!(arguments(&args), Err(USAGE), "{tail:?}");
        }
        for prefix in ["/", "relative/", "/missing-end", "/a//b/", "/../"] {
            let args =
                ["--module", "notes.wasm", "--name", "notes", "--prefix", prefix, "--out", "out"].map(str::to_owned);
            assert_eq!(arguments(&args), Err(USAGE), "{prefix}");
        }
        assert_eq!(arguments(&[]), Err(USAGE));
    }
}
