use pprint::core::combinator::{line, text};
use pprint::core::traits::{Doc, FlattenableDoc};

fn bracket(l: &str, x: impl FlattenableDoc, r: &str) -> impl FlattenableDoc {
    text(l)
        .cat(line().cat(x).nest(2))
        .cat_with_line(text(r))
        .group()
}

fn if_block(
    cond: impl FlattenableDoc,
    consq: impl FlattenableDoc,
    alter: impl FlattenableDoc,
) -> impl FlattenableDoc {
    text("if")
        .cat_with_space(bracket("(", cond, ")"))
        .cat_with_space(bracket("{", consq, "}"))
        .cat_with_space(text("else"))
        .cat_with_space(bracket("{", alter, "}"))
        .group()
}

fn main() {
    let test = if_block(
        text("_"),
        if_block(text("_"), text("_"), text("_")),
        if_block(text("_"), text("_"), text("_")),
    );

    let w = 20;
    println!("{}", test.pretty(w));
}
