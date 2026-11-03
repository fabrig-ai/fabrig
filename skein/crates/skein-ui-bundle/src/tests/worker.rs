//! Worker modules beside a page's module (playground.md, 7).

use super::{contents, glue, inputs};
use crate::{Refusal, WORKER_STARTER, Worker, bundle, bundle_with};
use skein_scratch::Scratch;
use std::{fs, path::Path};

fn worker() -> Worker {
    let mut glue = glue();
    glue.js = b"export default function init() { postMessage('ready'); }".to_vec();
    glue.wasm = b"\0asm\x01\0\0\0".to_vec();
    glue.snippets = vec![("world/grow.js".into(), b"export function grow() { return 1; }".to_vec())];
    Worker { name: "world".into(), glue }
}

#[test]
fn a_worker_is_bundled_beside_the_page() {
    let scratch = Scratch::new("bundle-worker-files");
    let inputs = inputs(&scratch);
    let worker = worker();
    let plain = scratch.path().join("plain");
    let out = scratch.path().join("worker");
    let plain_digest = bundle(&inputs, &glue(), &plain).expect("page bundle");
    let digest = bundle_with(&inputs, &glue(), Some(&worker), &out).expect("worker bundle");
    let files = contents(&out.join(digest.to_string()));
    assert_eq!(
        files.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "a.css",
            "boot-worker.js",
            "boot.js",
            "files/about.txt",
            "notes.js",
            "notes_bg.wasm",
            "pictures/notes/icon.svg",
            "skein-ui.css",
            "snippets/notes/grow.js",
            "snippets/world/grow.js",
            "world.js",
            "world_bg.wasm",
        ]
    );
    let mut expected = contents(&plain.join(plain_digest.to_string()));
    expected.insert(WORKER_STARTER.into(), b"import init from \"./world.js\";\ninit();\n".to_vec());
    expected.insert("world.js".into(), worker.glue.js);
    expected.insert("world_bg.wasm".into(), worker.glue.wasm);
    for (path, bytes) in worker.glue.snippets {
        expected.insert(format!("snippets/{path}"), bytes);
    }
    assert_eq!(files, expected);
    let plain_page = fs::read_to_string(plain.join("index.html")).expect("plain page");
    let page = fs::read_to_string(out.join("index.html")).expect("worker page");
    let plain_directory = format!("{}{plain_digest}/", inputs.prefix);
    let directory = format!("{}{digest}/", inputs.prefix);
    assert!(plain_page.contains(&plain_directory));
    assert!(page.contains(&directory));
    assert_eq!(plain_page.replace(&plain_directory, "{directory}/"), page.replace(&directory, "{directory}/"));
}

#[test]
fn a_worker_changes_the_digest() {
    let scratch = Scratch::new("bundle-worker-digest");
    let inputs = inputs(&scratch);
    let worker = worker();
    let out = scratch.path().join("out");
    let plain = bundle(&inputs, &glue(), &out).expect("page bundle");
    let first = bundle_with(&inputs, &glue(), Some(&worker), &out).expect("worker bundle");
    assert_ne!(plain, first);
    let files = contents(&out);
    let second = bundle_with(&inputs, &glue(), Some(&worker), &out).expect("same worker bundle");
    assert_eq!(first, second);
    assert_eq!(files, contents(&out));

    let mut renamed = worker.clone();
    renamed.name = "other".into();
    let mut js = worker.clone();
    js.glue.js.push(b'\n');
    let mut wasm = worker.clone();
    wasm.glue.wasm.push(0);
    let mut snippet_path = worker.clone();
    snippet_path.glue.snippets = vec![("world/other.js".into(), b"export function grow() { return 1; }".to_vec())];
    let mut snippet_bytes = worker.clone();
    snippet_bytes.glue.snippets = vec![("world/grow.js".into(), b"export function grow() { return 2; }".to_vec())];
    for changed in [renamed, js, wasm, snippet_path, snippet_bytes] {
        let digest = bundle_with(&inputs, &glue(), Some(&changed), &out).expect("changed worker bundle");
        assert_ne!(first, digest, "{changed:?}");
    }
}

#[test]
fn a_bundle_without_a_worker_is_unchanged() {
    let scratch = Scratch::new("bundle-no-worker");
    let inputs = inputs(&scratch);
    let a = scratch.path().join("a");
    let b = scratch.path().join("b");
    let first = bundle(&inputs, &glue(), &a).expect("bundle");
    let second = bundle_with(&inputs, &glue(), None, &b).expect("bundle without a worker");
    assert_eq!(first, second);
    assert_eq!(contents(&a), contents(&b));
}

#[test]
fn a_shared_snippet_is_kept_once() {
    let scratch = Scratch::new("bundle-shared-snippet");
    let inputs = inputs(&scratch);
    let mut worker = worker();
    let out = scratch.path().join("out");
    let original = bundle_with(&inputs, &glue(), Some(&worker), &out).expect("distinct snippets");
    let files = contents(&out);
    worker.glue.snippets.extend(glue().snippets);
    let shared = bundle_with(&inputs, &glue(), Some(&worker), &out).expect("shared snippet");
    assert_eq!(original, shared, "a shared snippet adds no file");
    assert_eq!(files, contents(&out), "a shared snippet is kept unchanged");
}

#[test]
fn a_clashing_snippet_is_refused() {
    let scratch = Scratch::new("bundle-clashing-snippet");
    let inputs = inputs(&scratch);
    let mut worker = worker();
    worker.glue.snippets.push(("notes/grow.js".into(), b"different bytes".to_vec()));
    let out = scratch.path().join("out");
    bundle(&inputs, &glue(), &out).expect("existing bundle");
    let files = contents(&out);
    assert_eq!(
        bundle_with(&inputs, &glue(), Some(&worker), &out),
        Err(Refusal::SameName { file: Path::new("notes/grow.js").into() })
    );
    assert_eq!(files, contents(&out), "refusal preserves the existing output");
}

#[test]
fn a_worker_named_as_the_page_is_refused() {
    let scratch = Scratch::new("bundle-worker-name");
    let inputs = inputs(&scratch);
    let mut worker = worker();
    worker.name.clone_from(&inputs.name);
    assert_eq!(
        bundle_with(&inputs, &glue(), Some(&worker), &scratch.path().join("out")),
        Err(Refusal::SameName { file: Path::new("notes.js").into() })
    );
}
