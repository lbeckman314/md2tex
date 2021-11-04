use std::fmt::Write;

#[derive(Debug)]
pub struct TexWriter<T: Write> {
    buffer: T,
}

impl<T: Write> TexWriter<T> {
    pub fn new(buffer: T) -> Self {
        Self { buffer }
    }

    pub fn new_line(&mut self) -> &mut Self {
        self.buffer.write_char('\n').unwrap();
        self
    }

    pub fn back_slash(&mut self) -> &mut Self {
        self.buffer.write_char('\\').unwrap();
        self
    }

    pub fn push_str(&mut self, s: &str) -> &mut Self {
        self.buffer.write_str(s).unwrap();
        self
    }

    pub fn escape_str(&mut self, s: &str) -> &mut Self {
        let escaped = escape_tex_text(s);
        self.push_str(&escaped)
    }

    pub fn push(&mut self, c: char) -> &mut Self {
        self.buffer.write_char(c).unwrap();
        self
    }

    pub fn push_lines<'a>(&mut self, iter: impl IntoIterator<Item = &'a str>) -> &mut Self {
        for s in iter {
            self.push_str(s).new_line();
        }

        self
    }

    pub fn buffer(&mut self) -> &mut T {
        &mut self.buffer
    }

    pub fn into_buffer(self) -> T {
        self.buffer
    }
}

impl<'a, T: Write> Extend<&'a str> for TexWriter<T> {
    fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iter: I) {
        for s in iter {
            self.push_str(s);
        }
    }
}

impl<T: Write> Extend<char> for TexWriter<T> {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        for c in iter {
            self.push(c);
        }
    }
}

impl<T: Write> Write for TexWriter<T> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buffer.write_str(s)
    }
}

fn escape_tex_text(md: &str) -> String {
    md.replace(r"\", r"\\")
        .replace("&", r"\&")
        .replace(r"\s", r"\textbackslash{}s")
        .replace(r"\w", r"\textbackslash{}w")
        .replace("_", r"\_")
        .replace(r"\<", "<")
        .replace(r"%", r"\%")
        .replace(r"$", r"\$")
        .replace(r"—", "---")
        .replace("#", r"\#")
}
