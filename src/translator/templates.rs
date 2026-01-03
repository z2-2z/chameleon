use crate::translator::TranslatorGrammar;
use askama::Template;

#[derive(askama::Template)]
#[template(path = "mutations.c", escape = "none")]
struct Mutations<'a> {
    grammar: &'a TranslatorGrammar,
}

#[derive(askama::Template)]
#[template(path = "numbersets.c", escape = "none")]
struct Numbersets<'a> {
    grammar: &'a TranslatorGrammar,
}

#[derive(askama::Template)]
#[template(path = "root.c", escape = "none")]
struct Root<'a> {
    grammar: &'a TranslatorGrammar,
    numbersets: Numbersets<'a>,
    mutations: Mutations<'a>,
}

pub fn render(grammar: TranslatorGrammar) -> String {
    let numbersets = Numbersets {
        grammar: &grammar,
    };
    let mutations = Mutations {
        grammar: &grammar,
    };
    let root = Root {
        grammar: &grammar,
        numbersets,
        mutations,
    };
    root.render().unwrap()
}
