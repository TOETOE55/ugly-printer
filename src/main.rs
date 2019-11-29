use typed_arena::Arena;
use std::borrow::Cow;
use std::rc::Rc;
use std::ops::Deref;
use pprint::core::basic::text;

#[derive(Clone)]
enum Tree {
    Static(usize, Box<Tree>, Box<Tree>),
    Dynamic(usize, Rc<dyn Fn() -> Tree>)
}

enum MaybeOwned<'a, T: ?Sized> {
    Brw(&'a T),
    Own(Box<T>),
}

impl<'a, T: ?Sized> Deref for MaybeOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Brw(x) => x,
            MaybeOwned::Own(x) => x,
        }
    }
}



fn traverse(tree: &Tree) -> Vec<usize> {
    let mut linear = vec![];
    let mut stack = vec![Cow::Borrowed(tree)];
    // let arena = Arena::new();
    while let Some(node) = stack.pop() {
        match node {
            Cow::Borrowed(Tree::Static(n, x, y)) => {
                linear.push(*n);
                stack.push(Cow::Borrowed(y));
                stack.push(Cow::Borrowed(x));
            },
            Cow::Borrowed(Tree::Dynamic(n, f)) => {
                linear.push(*n);
                stack.push(Cow::Owned(f()));
            },
            Cow::Owned(Tree::Static(n, x, y)) => {
                linear.push(n);
                stack.push(Cow::Owned(*y));
                stack.push(Cow::Owned(*x));
            },
            Cow::Owned(Tree::Dynamic(n, f)) => {
                linear.push(n);
                stack.push(Cow::Owned(f()));
            }
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
    println!("{}", test.pretty(w));
}
