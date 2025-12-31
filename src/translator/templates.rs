use crate::translator::TranslatorGrammar;
use askama::Template;

#[derive(askama::Template)]
#[template(path = "root.c", escape = "none")]
struct Root {
    grammar: TranslatorGrammar,
}

pub fn render(grammar: TranslatorGrammar) -> String {
    let root = Root {
        grammar,
    };
    root.render().unwrap()
}
