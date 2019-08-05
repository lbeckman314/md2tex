use pulldown_cmark::{Event, Parser, Tag, Options};
use walkdir::WalkDir;
use inflector::cases::kebabcase::to_kebab_case;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;

pub const LATEX_BEGIN: &str = r#"
\begin{document}
\maketitle
\clearpage
\tableofcontents
\clearpage
"#;

pub const LATEX_FOOTER: &str = "\n\\end{document}\n";

#[derive(Debug)]
enum EventType {
    Code,
    Emphasis,
    Header,
    Strong,
    Text,
}

struct CurrentType {
    typ: EventType,
}

pub fn markdown_to_latex(markdown: String) -> String {
    let latex_header = include_str!("header.tex");
    let latex_languages = include_str!("languages.tex");

    let mut output = String::from(latex_header);
    output.push_str(&latex_languages);
    output.push_str(&LATEX_BEGIN);

    let mut header_value = String::new();

    let mut current_type: CurrentType = CurrentType{typ: EventType::Text};

    let mut options = Options::empty();
    options.insert(Options::FIRST_PASS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new(&markdown);

    for event in parser {
        println!("Event: {:?}", event);
        match event {
            Event::Start(Tag::Header(level)) => {
                current_type.typ = EventType::Header;
                output.push_str("\n");
                output.push_str("\\");
                match level {
                    -1 => output.push_str("part{"),
                    0 => output.push_str("chapter{"),
                    1 => output.push_str("section{"),
                    2 => output.push_str("subsection{"),
                    3 => output.push_str("subsubsection{"),
                    4 => output.push_str("paragraph{"),
                    5 => output.push_str("subparagraph{"),
                    _ => println!("header is out of range."),
                }
            },
            Event::End(Tag::Header(_)) => {
                output.push_str("}\n");
                output.push_str("\\");
                output.push_str("label{");
                output.push_str(&header_value);
                output.push_str("}\n");

                output.push_str("\\");
                output.push_str("label{");
                output.push_str(&to_kebab_case(&header_value));
                output.push_str("}\n");
            },
            Event::Start(Tag::Emphasis) => {
                current_type.typ = EventType::Emphasis;
                output.push_str("\\emph{");
            },
            Event::End(Tag::Emphasis) => output.push_str("}"),

            Event::Start(Tag::Strong) => {
                current_type.typ = EventType::Strong;
                output.push_str("\\textbf{");
            },
            Event::End(Tag::Strong) => output.push_str("}"),

            Event::Start(Tag::List(None)) => output.push_str("\\begin{itemize}\n"),
            Event::End(Tag::List(None)) => output.push_str("\\end{itemize}\n"),

            Event::Start(Tag::List(Some(_))) => output.push_str("\\begin{enumerate}\n"),
            Event::End(Tag::List(Some(_))) => output.push_str("\\end{enumerate}\n"),

            Event::Start(Tag::Link(_, url, _)) => {
                if url.starts_with("http") {
                    output.push_str("\\href{");
                    output.push_str(&*url);
                    output.push_str("}{");
                }
                else {
                    output.push_str("\\hyperref[");
                    let mut found = false;

                    for entry in WalkDir::new("../../src").into_iter().filter_map(|e| e.ok()) {
                        let _path = entry.path().to_str().unwrap();
                        let _url = &url.clone().into_string().replace("../", "");
                        if _path.ends_with(_url) {
                            println!("{}", entry.path().display());
                            println!("URL: {}", url);

                            let file = match fs::File::open(_path) {
                                Ok(file) => file,
                                Err(_) => panic!("Unable to read title from {}", _path),
                            };
                            let buffer = BufReader::new(file);

                            let title = title_string(buffer);
                            output.push_str(&title);

                            println!("The title is '{}'", title);

                            found = true;
                            break;
                        }
                    }

                    if !found {
                        output.push_str(&*url.replace("#", ""));
                    }

                    output.push_str("]{");
                    }
            },

            Event::End(Tag::Link(_, _, _)) => {
                output.push_str("}");
            },

            Event::Start(Tag::Image(_, path, title)) => {
                output.push_str("\\begin{figure}\n");
                output.push_str("\\centering\n");
                output.push_str("\\includegraphics[width=\\textwidth]{");;
                output.push_str(&*path);
                output.push_str("}\n");
                output.push_str("\\caption{");
                output.push_str(&*title);
                output.push_str("}\n\\end{figure}\n");
            },

            Event::Start(Tag::Item) => output.push_str("\\item "),
            Event::End(Tag::Item) => output.push_str("\n"),

            Event::Start(Tag::CodeBlock(lang)) => {
                current_type.typ = EventType::Code;
                if ! lang.is_empty() {
                    output.push_str("\\begin{lstlisting}[language=");
                    output.push_str(&*lang.replace(",editable", ""));
                    output.push_str("]\n");
                } else {
                    output.push_str("\\begin{lstlisting}\n");
                }
            },

            Event::End(Tag::CodeBlock(_)) => {
                output.push_str("\n\\end{lstlisting}\n");
                current_type.typ = EventType::Text;
            },

            Event::Code(t) => {
                output.push_str("\\lstinline|");
                output.push_str(&*t);
                output.push_str("|");
            },

            Event::Text(t) => {
                println!("current_type: {:?}", current_type.typ);
                match current_type.typ {
                    EventType::Strong | EventType::Emphasis | EventType::Text  => {
                        output.push_str(&*t.replace("&", "\\&")
                            .replace("_", "\\_")
                            .replace("\\<", "<")
                            .replace("#", "\\#"));
                    },
                    EventType::Header => {
                        output.push_str(&*t.replace("&", "\\&")
                            .replace("_", "\\_")
                            .replace("\\<", "<")
                            .replace("#", "\\#"));
                        header_value = t.into_string();
                    },
                    _ => {
                        output.push_str(&*t);
                    },
                }
            },

            Event::SoftBreak => {
                output.push('\n');
            },

            _ => (),
        }
    }

    output.push_str(LATEX_FOOTER);

    output
}

pub fn markdown_to_pdf(markdown: String) -> Result<Vec<u8>, tectonic::Error> {
    tectonic::latex_to_pdf(markdown_to_latex(markdown))
}

/// Get the title of a Markdown file.
///
/// Reads the first line of a Markdown file, strips any hashes and
/// leading/trailing whitespace, and returns the title.
/// https://codereview.stackexchange.com/questions/135013/rust-function-to-read-the-first-line-of-a-file-strip-leading-hashes-and-whitesp
fn title_string<R>(mut rdr: R) -> String
where R: BufRead,
{
    let mut first_line = String::new();

    rdr.read_line(&mut first_line).expect("Unable to read line");

    // Where do the leading hashes stop?
    let last_hash = first_line
        .char_indices()
        .skip_while(|&(_, c)| c == '#')
        .next()
        .map_or(0, |(idx, _)| idx);

    // Trim the leading hashes and any whitespace
    first_line[last_hash..].trim().into()
}
