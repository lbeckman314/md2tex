use md2tex::Converter;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[test]
fn integration_test() {
    let content_file = "tests/book.md";
    let content = read_to_string(content_file).expect("Something went wrong reading the file");
    let path = Path::new("tests/book.tex");
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let template_file = "tests/template.tex";
    let template = read_to_string(template_file).expect("Something went wrong reading the file");

    let latex = Converter::new(&content)
        .template(&template)
        .assets(Path::new("tests/book/src/"))
        .run();

    match file.write_all(latex.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}
