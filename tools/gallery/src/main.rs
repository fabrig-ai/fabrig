use std::{env, path::Path, process};
fn main() {
    skein_fail_stop::install();
    let mut args = env::args_os();
    let _program = args.next();
    let Some(out) = args.next() else {
        eprintln!("usage: temper-client-gallery OUT");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("usage: temper-client-gallery OUT");
        process::exit(2);
    }
    if let Err(why) = temper_client_gallery::print(Path::new(&out)) {
        eprintln!("temper-client-gallery: {why:?}");
        process::exit(1);
    }
    for sample in temper_client_gallery::samples::samples() {
        println!("{}", sample.name);
    }
}
