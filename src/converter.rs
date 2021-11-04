use std::path::Path;

pub struct Converter<'a> {
    pub(crate) content: &'a str,
    pub(crate) template: Option<&'a str>,
    pub(crate) assets: Option<&'a Path>,
    pub(crate) chap_offset: i32,
    pub(crate) code_utf8_escape: Option<(&'a str, &'a str)>,
}

impl<'a> Converter<'a> {
    pub fn new(content: &'a str) -> Converter<'a> {
        Converter {
            content,
            template: None,
            assets: None,
            chap_offset: 0,
            code_utf8_escape: None,
        }
    }

    pub fn template(self, template: &'a str) -> Converter {
        Converter {
            template: Some(template),
            ..self
        }
    }

    pub fn assets(self, assets: &'a Path) -> Converter {
        Converter {
            assets: Some(assets),
            ..self
        }
    }

    pub fn chapter_level_offset(mut self, offset: i32) -> Self {
        self.chap_offset = offset;
        self
    }

    /// Set escape for UTF-8 characters inside code listings
    pub fn code_utf8_escape(mut self, start_escape: &'a str, end_escape: &'a str) -> Self {
        self.code_utf8_escape = Some((start_escape, end_escape));
        self
    }

    pub fn run(self) -> String {
        let latex = super::convert(&self);

        let mut output = String::new();
        match self.template {
            Some(template) => {
                output.push_str(template);
                // Insert new LaTeX data into template after "\begin{document}".
                let mark = "\\begin{document}";
                let pos = template.find(&mark).unwrap() + mark.len();
                output.insert_str(pos, &latex);
            }
            None => output.push_str(&latex),
        }

        output
    }
}
