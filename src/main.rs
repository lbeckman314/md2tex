use md2tex::markdown_to_latex;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

fn main() -> io::Result<()> {
    let mut f = File::open(Path::new(&env::args().nth(1).unwrap()))?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;

    println!("{}", markdown_to_latex(buffer));
    Ok(())
}
