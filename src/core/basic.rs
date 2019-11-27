#[derive(Clone, Debug)]
pub enum Doc {
    Nil,
    Line,
    Nest(usize, Box<Self>),
    Text(String),
    Cat(Box<Self>, Box<Self>),
    Union(Box<Self>, Box<Self>),
    FlatAt(Box<Self>, Box<Self>),
}

use Doc::*;
use crate::core::traits::{SimpleDoc, SimpleDocElem};

pub fn nil() -> Doc {
    Nil
}

pub fn text(txt: &str) -> Doc {
    Text(txt.to_string())
}

pub fn space() -> Doc {
    text(" ")
}

pub fn line() -> Doc {
    FlatAt(Box::new(Line), Box::new(space()))
}

pub fn hard_line() -> Doc {
    Line
}

impl Doc {
    pub fn nest(self, nested: usize) -> Self {
        Nest(nested, Box::new(self))
    }

    pub fn cat(self, rhs: Self) -> Self {
        Cat(Box::new(self), Box::new(rhs))
    }

    pub fn cat_with_line(self, rhs: Self) -> Self {
        self.cat(line()).cat(rhs)
    }

    pub fn cat_with_space(self, rhs: Self) -> Self {
        self.cat(space()).cat(rhs)
    }

    pub fn group(self) -> Self {
        Union(Box::new(self.clone().flatten()), Box::new(self))
    }

    pub fn flatten(self) -> Self {
        match self {
            Nest(nested, x) => x.flatten().nest(nested),
            Cat(x, y) => x.flatten().cat(y.flatten()),
            FlatAt(_, y) => *y,
            Union(x, _) => x.flatten(),
            other => other,
        }
    }

    pub fn pretty(&self, w: usize) -> String {
        let mut linear = SimpleDoc::default();
        let mut stack = vec![(0, self)];
        be(w, 0, 0, &mut stack, &mut linear);
        format!("{}", linear)
    }
}


fn be<'a>(
    w: usize,
    mut row: usize,
    mut col: usize,
    stack: &mut Vec<(usize, &'a Doc)>,
    linear: &mut SimpleDoc<'a>,
) {
    while let Some((nested, doc)) = stack.pop() {
        match doc {
            Doc::Nil => {}
            Doc::Line => {
                linear.0.push(SimpleDocElem::Line(nested));
                row += 1;
            }
            Doc::Text(txt) => {
                linear.0.push(SimpleDocElem::Text(txt));
                col += txt.len();
            }
            Doc::Cat(x, y) => {
                stack.push((nested, y));
                stack.push((nested, x));
            }
            Doc::Nest(j, x) => {
                stack.push((nested + *j, x));
                col = nested;
            }
            Doc::FlatAt(x, _) => stack.push((nested, x)),
            Doc::Union(x, y) => {
                let stack_saved = stack.clone();
                let linear_saved = linear.clone();

                stack.push((nested, x));
                be(w, row, col, stack, linear);
                if linear.fits(w - col) {
                    break;
                } else {
                    *stack = stack_saved;
                    *linear = linear_saved;
                    stack.push((nested, y));
                }
            }
        }
    }
}
