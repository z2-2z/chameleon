use crate::translator::TranslatorGrammar;
use tinytemplate::TinyTemplate;

static ROOT: &str = include_str!("../templates/root.c");

pub fn render(grammar: TranslatorGrammar) -> String {
    let mut renderer = TinyTemplate::new();
    renderer.add_template("root", ROOT).unwrap();
    renderer.render("root", &grammar).unwrap()
}
