use std::rc::Rc;
use pprint::core::basic::{Doc, text, nil, line, soft_line, line_break};

fn bracket(l: &str, x: Doc, r: &str) -> Doc {
    text(l).cat(line().cat(x).nest(2)).cat_with_line(text(r)).group()
}

fn if_block(cond: Doc, consq: Doc, alter: Doc) -> Doc {
    text("if").cat_with_space(bracket("(", cond, ")"))

        .cat_with_space(bracket("{", consq, "}"))
        .cat_with_space(text("else"))

        .cat_with_space(bracket("{", alter, "}"))
        .group()
}

fn main() {
    let test = if_block(
        text("_"),
        if_block(
            text("_"),
            text("_"),
            text("_")),
        if_block(
            text("_"),
            text("_"),
            text("_")));

    let w = 10;
    println!("{}", test.pretty(w));
}
