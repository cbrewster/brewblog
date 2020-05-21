use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};

pub struct MarkdownRenderer {
    theme_set: ThemeSet,
    syntax_set: SyntaxSet,
}

impl MarkdownRenderer {
    pub fn new() -> MarkdownRenderer {
        let syntax_set = SyntaxSet::load_defaults_newlines();

        let mut theme_set = ThemeSet::load_defaults();
        let one_dark_data = include_str!("../assets/themes/one-dark.tmTheme");
        let one_dark_theme =
            ThemeSet::load_from_reader(&mut std::io::Cursor::new(one_dark_data)).unwrap();

        theme_set.themes.insert("one-dark".into(), one_dark_theme);

        MarkdownRenderer {
            theme_set,
            syntax_set,
        }
    }

    pub fn render(&self, markdown: &str) -> String {
        let opts = Options::all();
        let mut s = String::with_capacity(&markdown.len() * 3 / 2);
        let p = Parser::new_ext(&markdown, opts);

        // TODO: Allow this to be configured.
        let theme = &self.theme_set.themes["one-dark"];

        // We'll build a new vector of events since we can only consume the parser once
        let mut new_p = Vec::new();
        // As we go along, we'll want to highlight code in bundles, not lines
        let mut to_highlight = String::new();
        // And track a little bit of state
        let mut syntax_highligher = None;

        for event in p {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref syntax))) => {
                    if !syntax.is_empty() {
                        syntax_highligher =
                            self.syntax_set.find_syntax_by_extension(syntax.as_ref());
                    }
                    if syntax_highligher.is_none() {
                        new_p.push(event);
                    }
                }
                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    if let Some(syntax) = syntax_highligher {
                        // Format the whole multi-line code block as HTML all at once
                        let html = highlighted_html_for_string(
                            &to_highlight,
                            &self.syntax_set,
                            &syntax,
                            &theme,
                        );
                        // And put it into the vector
                        new_p.push(Event::Html(html.into()));
                        to_highlight = String::new();
                        syntax_highligher = None;
                    } else {
                        new_p.push(event);
                    }
                }
                Event::Text(t) => {
                    if syntax_highligher.is_some() {
                        // If we're in a code block, build up the string of text
                        to_highlight.push_str(&t);
                    } else {
                        new_p.push(Event::Text(t))
                    }
                }
                e => {
                    new_p.push(e);
                }
            }
        }

        // Now we send this new vector of events off to be transformed into HTML
        html::push_html(&mut s, new_p.into_iter());
        s
    }
}
