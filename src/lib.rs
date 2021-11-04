mod converter;
mod events;
mod writer;

use inflector::cases::kebabcase::to_kebab_case;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use regex::Regex;
use std::default::Default;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;
use tiny_skia::Pixmap;
use walkdir::WalkDir;
use writer::TexWriter;

use events::*;
use log::*;

pub use converter::Converter;

/// TODO https://github.com/raphlinus/pulldown-cmark/blob/master/src/html.rs

/// Backwards-compatible function.
pub fn markdown_to_tex(content: String) -> String {
    Converter::new(&content).run()
}

/// Converts markdown string to tex string.
fn convert(converter: &Converter) -> String {
    log::info!("Start conversion");

    let mut writer = TexWriter::new(String::new());

    let mut header_value = String::new();
    let mut table_buffer = TexWriter::new(String::new());

    let mut event_stack = Vec::new();

    let mut cells = 0;

    let options = Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES;

    let parser = Parser::new_ext(converter.content, options);

    let mut buffer = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level)) => {
                let last_ev = event_stack.last().copied().unwrap_or_default();
                let level = level as i32 + converter.chap_offset;

                event_stack.push(EventType::Header);

                writer.new_line();
                match level {
                    i32::MIN..=0 => writer.push_str(r"\chapter{"),
                    1 => writer.push_str(r"\section{"),
                    2 => writer.push_str(r"\subsection{"),
                    3 => writer.push_str(r"\subsubsection{"),
                    4 => {
                        // https://tex.stackexchange.com/questions/169830/pdflatex-raise-error-when-paragraph-inside-quote-environment
                        if matches!(last_ev, EventType::BlockQuote) {
                            writer.push_str(r"\mbox{} %").new_line();
                        }
                        writer.push_str(r"\paragraph{")
                    }
                    5..=i32::MAX => {
                        // https://tex.stackexchange.com/questions/169830/pdflatex-raise-error-when-paragraph-inside-quote-environment
                        if matches!(last_ev, EventType::BlockQuote) {
                            writer.push_str(r"\mbox{} %").new_line();
                        }
                        writer.push_str(r"\subparagraph{")
                    }
                };
            }
            Event::End(Tag::Heading(_)) => {
                writeln!(
                    writer,
                    "}}\n\\label{{{}}}\n\\label{{{}}}",
                    header_value,
                    to_kebab_case(&header_value)
                )
                .unwrap();

                event_stack.pop();
            }
            Event::Start(Tag::Emphasis) => {
                event_stack.push(EventType::Emphasis);
                writer.push_str(r"\emph{");
            }
            Event::End(Tag::Emphasis) => {
                writer.push('}');
                event_stack.pop();
            }

            Event::Start(Tag::Strong) => {
                event_stack.push(EventType::Strong);
                writer.push_str(r"\textbf{");
            }
            Event::End(Tag::Strong) => {
                writer.push('}');
                event_stack.pop();
            }

            Event::Start(Tag::BlockQuote) => {
                event_stack.push(EventType::BlockQuote);
                writer.new_line().push_str(r"\begin{quote}").new_line();
            }
            Event::End(Tag::BlockQuote) => {
                writer
                    .new_line()
                    .push_str(r"\end{quote}")
                    .new_line()
                    .new_line();

                event_stack.pop();
            }

            Event::Start(Tag::List(None)) => {
                writer.new_line().push_str(r"\begin{itemize}").new_line();
            }
            Event::End(Tag::List(None)) => {
                writer.new_line().push_str(r"\end{itemize}").new_line();
            }

            Event::Start(Tag::List(Some(_))) => {
                writer.push_str(r"\begin{enumerate}").new_line();
            }
            Event::End(Tag::List(Some(_))) => {
                writer.push_str(r"\end{enumerate}").new_line();
            }

            Event::Start(Tag::Paragraph) => {
                writer.new_line();
            }

            Event::End(Tag::Paragraph) => {
                // ~ adds a space to prevent
                // "There's no line here to end" error on empty lines.
                writer.push_str(r"~\\").new_line();
            }

            Event::Start(Tag::Link(_, url, _)) => {
                // URL link (e.g. "https://nasa.gov/my/cool/figure.png")
                if url.starts_with("http") {
                    write!(writer, r"\href{{{}}}{{", url).unwrap();
                // local link (e.g. "my/cool/figure.png")
                } else {
                    writer.push_str(r"\hyperref[");
                    let mut found = false;

                    // iterate through `src` directory to find the resource.
                    for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
                        let _path = entry.path().to_str().unwrap();
                        let _url = &url.clone().into_string().replace("../", "");
                        if _path.ends_with(_url) {
                            match fs::File::open(_path) {
                                Ok(_) => (),
                                Err(_) => panic!("Unable to read title from {}", _path),
                            };

                            found = true;
                            break;
                        }
                    }

                    if !found {
                        writer.push_str(&*url.replace("#", ""));
                    }

                    writer.push_str("]{");
                }
            }

            Event::End(Tag::Link(_, _, _)) => {
                writer.push('}');
            }

            Event::Start(Tag::Table(_)) => {
                event_stack.push(EventType::Table);

                let table_start = [
                    r"\begingroup",
                    r"\setlength{\LTleft}{-20cm plus -1fill}",
                    r"\setlength{\LTright}{\LTleft}",
                    r"\begin{longtable}{!!!}",
                    r"\hline",
                    r"\hline",
                ];

                table_buffer.new_line().push_lines(table_start).new_line();
            }

            Event::End(Tag::Table(_)) => {
                let table_end = [
                    r"\arrayrulecolor{black}\hline",
                    r"\end{longtable}",
                    r"\endgroup",
                ];

                table_buffer.push_lines(table_end).new_line();

                let mut cols = String::new();
                for _i in 0..cells {
                    write!(cols, r"C{{{width}\textwidth}} ", width = 1. / cells as f64).unwrap();
                }

                writer.push_str(&table_buffer.buffer().replace("!!!", &cols));
                table_buffer.buffer().clear();

                cells = 0;

                event_stack.pop();
            }

            Event::Start(Tag::TableHead) => {
                event_stack.push(EventType::TableHead);
            }

            Event::End(Tag::TableHead) => {
                let limit = table_buffer.buffer().len() - 2;

                table_buffer.buffer().truncate(limit);

                table_buffer
                    .back_slash()
                    .back_slash()
                    .new_line()
                    .push_str(r"\hline")
                    .new_line();

                event_stack.pop();
            }

            Event::Start(Tag::TableCell) => {
                if matches!(event_stack.last(), Some(EventType::TableHead)) {
                    table_buffer.push_str(r"\bfseries{");
                }
            }

            Event::End(Tag::TableCell) => {
                if matches!(event_stack.last(), Some(EventType::TableHead)) {
                    table_buffer.push('}');
                    cells += 1;
                }

                table_buffer.push_str(" & ");
            }

            Event::Start(Tag::TableRow) => {}

            Event::End(Tag::TableRow) => {
                let limit = table_buffer.buffer().len() - 2;

                table_buffer.buffer().truncate(limit);

                table_buffer
                    .push_str(r"\\\arrayrulecolor{lightgray}\hline")
                    .new_line();
            }

            Event::Start(Tag::Image(_, path, title)) => {
                let mut assets_path = converter
                    .assets
                    .map(|p| p.to_path_buf())
                    .unwrap_or_default()
                    .join(path.as_ref());

                let mut path = PathBuf::from_str(path.as_ref()).unwrap();

                // if image path ends with ".svg", run it through
                // svg2png to convert to png file.
                if path.extension().unwrap() == "svg" {
                    let img = svg2png(&assets_path);

                    path.set_extension("png");
                    let path = path
                        .strip_prefix("../..")
                        .map(Path::to_path_buf)
                        .unwrap_or(path);

                    // create output directories.
                    let _ = fs::create_dir_all(path.parent().unwrap());

                    img.save_png(&path).unwrap();
                    assets_path = path;
                }

                writer
                    .push_str(r"\begin{figure}")
                    .new_line()
                    .push_str(r"\centering")
                    .new_line()
                    .push_str(r"\includegraphics[width=\textwidth]{")
                    .push_str(assets_path.to_string_lossy().as_ref())
                    .push('}')
                    .new_line()
                    .push_str(r"\caption{")
                    .push_str(&*title)
                    .push('}')
                    .new_line()
                    .push_str(r"\end{figure}")
                    .new_line();
            }

            Event::Start(Tag::Item) => {
                writer.push_str(r"\item ");
            }
            Event::End(Tag::Item) => {
                writer.new_line();
            }

            Event::Start(Tag::CodeBlock(lang)) => {
                let re = Regex::new(r",.*").unwrap();

                match lang {
                    CodeBlockKind::Indented => {
                        writer.push_str(r"\begin{lstlisting}").new_line();
                    }
                    CodeBlockKind::Fenced(lang) => {
                        writer.push_str(r"\begin{lstlisting}[language=");
                        let lang = re.replace(&lang, "");
                        let lang = lang
                            .split_whitespace()
                            .next()
                            .unwrap_or_else(|| lang.as_ref());

                        writeln!(writer, "{}]", lang).unwrap();
                    }
                }

                event_stack.push(EventType::Code);
            }

            Event::End(Tag::CodeBlock(_)) => {
                writer.new_line().push_str(r"\end{lstlisting}").new_line();

                event_stack.pop();
            }

            Event::Code(t) => {
                let wr = if event_stack
                    .iter()
                    .any(|ev| matches!(ev, EventType::Table | EventType::TableHead))
                {
                    &mut table_buffer
                } else {
                    &mut writer
                };

                if event_stack.contains(&EventType::Header) {
                    wr.push_str(r"\texttt{").escape_str(&t).push('}');
                } else {
                    let mut code = String::with_capacity(t.len());

                    if let Some((es, ee)) = converter.code_utf8_escape {
                        for c in t.chars() {
                            if c.is_ascii() {
                                code.push(c);
                            } else {
                                write!(code, "{}{}{}", es, c, ee).unwrap();
                            }
                        }
                    } else {
                        code.push_str(&*t);
                    }

                    let delims = ['|', '!', '?', '+', '@'];

                    let delim = delims
                        .iter()
                        .find(|c| !code.contains(**c))
                        .expect("Failed to find listing delmeter");

                    write!(wr, r"\lstinline{}{}{}", delim, code, delim).unwrap();
                }
            }

            Event::Html(_t) => {
                //current.event_type = EventType::Html;
                // convert common html patterns to tex
                //output.push_str(
                //convert(&parse_html(&t.into_string()), assets_prefix, chap_offset).as_str(),
                //);
                //current.event_type = EventType::Text;
            }

            Event::Text(t) => {
                // if "$$", "\[", "\(" are encountered, then begin equation
                // and don't replace any characters.

                let regex_eq_start = Regex::new(r"(\$\$|\\\[|\\\()").unwrap();
                let regex_eq_end = Regex::new(r"(\$\$|\\\]|\\\))").unwrap();

                buffer.clear();
                buffer.push_str(&t.to_string());

                let mut on_text = |wr: &mut TexWriter<String>| {
                    // TODO more elegant way to do ordered `replace`s (structs?).
                    while !buffer.is_empty() {
                        if let Some(m) = regex_eq_start.find(&buffer) {
                            log::debug!("Equation start: {:#?}", m);

                            let end = m.end();
                            let start = m.start();

                            wr.escape_str(&buffer[..start]).push_str(r"\[");
                            buffer.drain(..end);

                            let m = regex_eq_end
                                .find(&buffer)
                                .expect("Failed to detect end of equation");

                            log::debug!("Equation end: {:#?}", m);

                            let start = m.start();
                            let end = m.end();

                            wr.push_str(&buffer[..start]).push_str(r"\]");
                            buffer.drain(..end);
                        }

                        wr.escape_str(&buffer);
                        buffer.clear();

                        header_value = t.to_string();
                    }
                };

                match event_stack.last().copied().unwrap_or_default() {
                    EventType::Strong
                    | EventType::Emphasis
                    | EventType::Text
                    | EventType::Header => on_text(&mut writer),

                    EventType::Table | EventType::TableHead => on_text(&mut table_buffer),

                    _ => {
                        writer.push_str(&*t);
                    }
                }
            }

            Event::SoftBreak => {
                writer.new_line();
            }

            Event::HardBreak => {
                writer.back_slash().back_slash().new_line();
            }

            _ => (),
        }
    }

    writer.into_buffer()
}

/// Simple HTML parser.
///
/// Eventually I hope to use a mature 'HTML to tex' parser.
/// Something along the lines of https://github.com/Adonai/html2md/
/*
pub fn html2tex(html: String, current: &CurrentType, assets_prefix: Option<&Path>) -> String {
    let mut tex = html;
    let mut output = String::new();

    // remove all "class=foo" and "id=bar".
    let re = Regex::new(r#"\s(class|id)="[a-zA-Z0-9-_]*">"#).unwrap();
    tex = re.replace(&tex, "").to_string();

    // image html tags
    if tex.contains("<img") {
        // Regex doesn't yet support look aheads (.*?), so we'll use simple pattern matching.
        // let src = Regex::new(r#"src="(.*?)"#).unwrap();
        let src = Regex::new(r#"src="([a-zA-Z0-9-/_.]*)"#).unwrap();
        let caps = src.captures(&tex).unwrap();
        let path_raw = caps.get(1).unwrap().as_str();

        let mut path = assets_prefix
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
            .join(path_raw);

        // if path ends with ".svg", run it through
        // svg2png to convert to png file.
        if path.extension().unwrap().to_os_string() == "svg" {
            let img = svg2png(&path);
            path.set_extension("png");
            let path = path
                .strip_prefix("../../")
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| path.clone());

            // create output directories.
            let _ = fs::create_dir_all(Path::new(&path).parent().unwrap());

            img.save_png(std::path::Path::new(&path)).unwrap();
        }

        match current.event_type {
            EventType::Table => {
                output.push_str(r"\begin{center}\includegraphics[width=0.2\textwidth]{")
            }
            _ => {
                output.push_str(r"\begin{center}\includegraphics[width=0.8\textwidth]{");
            }
        }

        output.push_str(path.to_string_lossy().as_ref());
        output.push_str(r"}\end{center}\n");

    // all other tags
    } else {
        match current.event_type {
            // block code
            EventType::Html => {
                tex = tex
                    .replace("/>", "")
                    .replace("<code class=\"language-", "\\begin{lstlisting}")
                    .replace("</code>", r"\\end{lstlisting}")
                    .replace("<span", "")
                    .replace(r"</span>", "")
            }
            // inline code
            _ => {
                tex = tex
                    .replace("/>", "")
                    .replace("<code\n", "<code")
                    .replace("<code", r"\lstinline|")
                    .replace("</code>", r"|")
                    .replace("<span", "")
                    .replace(r"</span>", "");
            }
        }
        // remove all HTML comments.
        let re = Regex::new(r"<!--.*-->").unwrap();
        output.push_str(&re.replace(&tex, ""));
    }

    output
}
*/

/// Converts an SVG file to a PNG file.
///
/// Example: foo.svg becomes foo.svg.png
pub fn svg2png(filename: &Path) -> Pixmap {
    debug!("svg2png path: {:?}", &filename);
    let opt = usvg::Options::default();
    let svg_data = std::fs::read(filename).unwrap();
    let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

    let pixmap_size = rtree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&rtree, usvg::FitTo::Original, pixmap.as_mut()).unwrap();

    pixmap
}

