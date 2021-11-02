use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

use md2tex::markdown_to_tex;

macro_rules! unwrap {
    ($e: expr, $m: expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}: {}", $m, e);
                exit(1);
            }
        }
    };
}

fn main() {
    let matches = App::new(crate_name!())
        .bin_name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("INPUT")
                .long("input")
                .short("i")
                .help("Input markdown files")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .long("output")
                .short("o")
                .help("Output tex or pdf file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let mut content = String::new();
    let mut input = unwrap!(
        File::open(matches.value_of("INPUT").unwrap()),
        "couldn't open input file"
    );
    unwrap!(
        input.read_to_string(&mut content),
        "couldn't read file content"
    );

    let output_path = matches.value_of("OUTPUT").unwrap();
    let mut output = unwrap!(File::create(output_path), "couldn't open output file");

    let tex = markdown_to_tex(content);
    output
        .write_all(tex.as_bytes())
        .expect("couldn't write output file");
}
