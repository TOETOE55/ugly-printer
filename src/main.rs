use typed_arena::Arena;
use pprint::core::combinator::text;
use pprint::core::traits::{Doc, FlattenableDoc, pretty};

enum Tree {
    Static(usize, Box<Tree>, Box<Tree>),
    Dynamic(usize, Box<dyn Fn() -> Tree>)
}

fn traverse(tree: &Tree) -> Vec<usize> {
    let mut linear = vec![];
    let mut stack = vec![tree];
    let arena = Arena::new();
    while let Some(node) = stack.pop() {
        match node {
            Tree::Static(n, x, y) => {
                linear.push(*n);
                stack.push(y);
                stack.push(x);
            },
            Tree::Dynamic(n, f) => {
                linear.push(*n);
                stack.push(arena.alloc(f()));
            },
        }
    }
    linear
}

fn main() {
    let test =
        text("hello")
        .cat_with_line(text("world"))
        .nest(2)
        .cat_with_line(text("!")).group();
    let w = 20;
    println!("{}", pretty(&test, w));
}
