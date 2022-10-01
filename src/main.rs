use std::env;
use std::ffi::OsString;

use pmv::{print_error, try_main};

fn main() -> Result<(), ()> {
    let args: Vec<OsString> = env::args_os().into_iter().collect();

    if let Err(err) = try_main(&args[..]) {
        print_error(err);
        return Err(());
    }

    Ok(())
}
