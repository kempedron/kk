use syntect::easy::HighlightLines;
use syntect::highlighting::FontStyle;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use termion::color;
use two_face::theme::EmbeddedThemeName;

pub struct Highlighter {
    ps: SyntaxSet,
    theme: syntect::highlighting::Theme,
    extension: String,
}

impl Highlighter {
    pub fn new(filename: &str) -> Self {
        let extension = filename.rsplit('.').next().unwrap_or("txt").to_string();
        let ps = two_face::syntax::extra_newlines();
        
        let found = ps.find_syntax_by_extension(&extension);
        eprintln!("Extension: {}, Syntax found: {}", extension, found.map(|s| s.name.as_str()).unwrap_or("NOT FOUND"));


        let theme = two_face::theme::extra()
            .get(EmbeddedThemeName::Dracula)
            .clone();
        Highlighter {
            ps,
            theme,
            extension,
        }
    }
    pub fn highlight_all(&self, lines: &[String]) -> Vec<String> {
        let syntax = self
            .ps
            .find_syntax_by_extension(&self.extension)
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let mut h = HighlightLines::new(syntax, &self.theme);
        let mut result = Vec::new();

        for line in lines {
            let text = format!("{}\n", line);
            let ranges = h.highlight_line(&text, &self.ps).unwrap_or_default();

            let mut colored = String::new();
            for (style, token) in ranges {

                let token = token.trim_end_matches('\n');
                if token.is_empty() { continue; }
                
                let fg = style.foreground;
                let bg = style.background;

                colored.push_str(&format!(
                    "{}",
                    termion::color::Bg(termion::color::Rgb(bg.r, bg.g, bg.b))
                ));
                colored.push_str(&format!(
                    "{}",
                    termion::color::Fg(termion::color::Rgb(fg.r, fg.g, fg.b))
                ));

                if style.font_style.contains(FontStyle::BOLD) {
                    colored.push_str(&format!("{}", termion::style::Bold));
                }
                if style.font_style.contains(FontStyle::ITALIC) {
                    colored.push_str(&format!("{}", termion::style::Italic));
                }
                if style.font_style.contains(FontStyle::UNDERLINE) {
                    colored.push_str(&format!("{}", termion::style::Underline));
                }

                colored.push_str(token);
            }
            colored.push_str(&format!(
                "{}{}",
                termion::style::Reset,
                termion::color::Bg(termion::color::Rgb(40, 42, 54))
            ));

            result.push(colored);
        }
        result
    }
}
