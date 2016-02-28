// build.rs
//
// Use this i oder to get the compiler verison at run time.
// (that is we want to know with which rust compiler the executable was built)


// Template taken from http://doc.crates.io/build-script.html

extern crate rustc_version;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use rustc_version::{version};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compiler_version.rs");
    let mut f = File::create(&dest_path).unwrap();

    write!(f, "pub const compiler_version: &'static str = \"{}\";\n", version()).expect(
    "I/O error writing compiler version");
}
