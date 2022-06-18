use std::env;
use std::ffi::OsString;

use pmv::{style_error, try_main};

fn main() -> Result<(), ()> {
    let args: Vec<OsString> = env::args_os().into_iter().map(|s| s.to_owned()).collect();

    if let Err(err) = try_main(&args[..]) {
        eprintln!("{}: {}", style_error("error"), err);
        return Err(());
    }

    Ok(())
}

