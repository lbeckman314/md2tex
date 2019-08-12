extern crate regex;

use inflector::cases::kebabcase::to_kebab_case;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use std::default::Default;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::iter::repeat;
use std::string::String;
use walkdir::WalkDir;

#[derive(Debug)]
enum EventType {
    Code,
    Emphasis,
    Header,
    Strong,
    TableHead,
    Text,
}

struct CurrentType {
    event_type: EventType,
}

pub fn markdown_to_latex(markdown: String) -> String {
    let mut output = String::new();

    let mut header_value = String::new();

    let mut current: CurrentType = CurrentType {
        event_type: EventType::Text,
    };
    let mut cells = 0;

    let mut options = Options::empty();
    options.insert(Options::FIRST_PASS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&markdown, options);

    for event in parser {
        println!("Event: {:?}", event);
        match event {
            Event::Start(Tag::Header(level)) => {
                current.event_type = EventType::Header;
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
            }
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
            }
            Event::Start(Tag::Emphasis) => {
                current.event_type = EventType::Emphasis;
                output.push_str("\\emph{");
            }
            Event::End(Tag::Emphasis) => output.push_str("}"),

            Event::Start(Tag::Strong) => {
                current.event_type = EventType::Strong;
                output.push_str("\\textbf{");
            }
            Event::End(Tag::Strong) => output.push_str("}"),

            Event::Start(Tag::List(None)) => output.push_str("\\begin{itemize}\n"),
            Event::End(Tag::List(None)) => output.push_str("\\end{itemize}\n"),

            Event::Start(Tag::List(Some(_))) => output.push_str("\\begin{enumerate}\n"),
            Event::End(Tag::List(Some(_))) => output.push_str("\\end{enumerate}\n"),

            Event::Start(Tag::Paragraph) => {
                output.push_str("\n");
            }

            Event::End(Tag::Paragraph) => {
                output.push_str(r"\\");
                output.push_str("\n");
            }

            Event::Start(Tag::Link(_, url, _)) => {
                if url.starts_with("http") {
                    output.push_str("\\href{");
                    output.push_str(&*url);
                    output.push_str("}{");
                } else {
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

                            println!("The`` title is '{}'", title);

                            found = true;
                            break;
                        }
                    }

                    if !found {
                        output.push_str(&*url.replace("#", ""));
                    }

                    output.push_str("]{");
                }
            }

            Event::End(Tag::Link(_, _, _)) => {
                output.push_str("}");
            }

            Event::Start(Tag::Table(_)) => {
                output.push_str("\n");
                output.push_str("\n");
                output.push_str(r"\begingroup");
                output.push_str("\n");
                output.push_str(r"\setlength{\LTleft}{-20cm plus -1fill}");
                output.push_str("\n");
                output.push_str(r"\setlength{\LTright}{\LTleft}");
                output.push_str("\n");
                output.push_str(r"\begin{longtable}{!!!}");
                output.push_str("\n");
                output.push_str(r"\hline");
                output.push_str("\n");
                output.push_str(r"\hline");
                output.push_str("\n");
            }

            Event::Start(Tag::TableHead) => {
                current.event_type = EventType::TableHead;
            }

            Event::End(Tag::TableHead) => {
                output.truncate(output.len() - 2);
                output.push_str(r"\\");
                output.push_str("\n");

                output.push_str(r"\hline");
                output.push_str("\n");
                current.event_type = EventType::Text;
            }

            Event::End(Tag::Table(_)) => {
                output.push_str("\n");
                output.push_str(r"\arrayrulecolor{black}\hline");
                output.push_str("\n");
                output.push_str(r"\end{longtable}");
                output.push_str("\n");
                output.push_str(r"\endgroup");
                output.push_str("\n");
                output.push_str("\n");
                let mut cols = String::new();
                for _i in 0..cells {
                    cols.push_str(&format!(
                        r"C{{{width}\textwidth}} ",
                        width = 1. / cells as f64
                    ));
                }
                output = output.replace("!!!", &cols);
                cells = 0;
            }

            Event::Start(Tag::TableCell) => match current.event_type {
                EventType::TableHead => {
                    output.push_str(r"\bfseries{");
                }
                _ => (),
            },

            Event::End(Tag::TableCell) => {
                match current.event_type {
                    EventType::TableHead => {
                        output.push_str(r"}");
                        cells += 1;
                    }
                    _ => (),
                }

                output.push_str(" & ");
            }

            Event::End(Tag::TableRow) => {
                output.truncate(output.len() - 2);
                output.push_str(r"\\");
                output.push_str(r"\arrayrulecolor{lightgray}\hline");
                output.push_str("\n");
            }

            Event::Start(Tag::Image(_, path, title)) => {
                output.push_str("\\begin{figure}\n");
                output.push_str("\\centering\n");
                output.push_str("\\includegraphics[width=\\textwidth]{");;
                output.push_str(&format!("../../src/{path}", path = path));
                output.push_str("}\n");
                output.push_str("\\caption{");
                output.push_str(&*title);
                output.push_str("}\n\\end{figure}\n");
            }

            Event::Start(Tag::Item) => output.push_str("\\item "),
            Event::End(Tag::Item) => output.push_str("\n"),

            Event::Start(Tag::CodeBlock(lang)) => {
                let re = Regex::new(r",.*").unwrap();
                current.event_type = EventType::Code;
                if !lang.is_empty() {
                    output.push_str("\\begin{lstlisting}[language=");
                    output.push_str(&re.replace(&lang, ""));
                    output.push_str("]\n");
                } else {
                    output.push_str("\\begin{lstlisting}\n");
                }
            }

            Event::End(Tag::CodeBlock(_)) => {
                output.push_str("\n\\end{lstlisting}\n");
                current.event_type = EventType::Text;
            }

            Event::Code(t) => {
                output.push_str("\\lstinline|");
                output.push_str(&*t.replace("…", "..."));
                output.push_str("|");
            }

            Event::InlineHtml(t) => {
                let mut latex = t.into_string();
                let re = Regex::new(r#"\s(class|id)=".*">"#).unwrap();
                latex = re.replace(&latex, "").to_string();

                if latex.contains("code>") {
                    latex = latex
                        .replace("<code>", r"\lstinline+")
                        .replace("</code>", r"+");
                }
                else if latex.contains("span>") {
                    latex = latex
                        .replace("<span class=\"caption\">", "")
                        .replace(r"</span>", "");
                }
//| <img src="img/ferris/does_not_compile.svg" class="ferris-explain"/>    | This code does not compile!                      |
                else if latex.contains("img>") {
                    latex = latex
                        .replace("<span class=\"caption\">", "")
                        .replace(r"</span>", "");
                }
                output.push_str(&latex);
            }

            Event::Html(t) => {
                let latex = t
                    .replace("</code>", r"\end{lstlisting}")
                    .replace("<code class=\"language-", r"\begin{lstlisting}[language=")
                    .replace("<pre>", "")
                    .replace("</pre>", "")
                    .replace("\">", "]");
                output.push_str(&latex);
            }

            Event::Text(t) => {
                println!("current_type: {:?}", current.event_type);
                match current.event_type {
                    EventType::Strong
                    | EventType::Emphasis
                    | EventType::Text
                    | EventType::Header => {
                        output.push_str(
                            &*t.replace("&", r"\&")
                                .replace("_", r"\_")
                                .replace(r"\<", "<")
                                .replace(r"%", "%")
                                .replace(r"$", r"\$")
                                .replace(r"—", "---")
                                .replace("#", r"\#"),
                        );
                        header_value = t.into_string();
                    }
                    _ => {
                        output.push_str(&*t);
                    }
                }
            }

            Event::SoftBreak => {
                output.push('\n');
            }

            Event::HardBreak => {
                output.push_str(r"\\");
                output.push('\n');
            }

            _ => (),
        }
    }

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
where
    R: BufRead,
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
