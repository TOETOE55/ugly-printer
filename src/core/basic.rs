
use Doc::*;
use std::rc::Rc;
use std::fmt::{Display, Formatter, Error};
use typed_arena::Arena;

#[derive(Clone)]
pub enum Doc<'a> {
    Nil,
    Line,
    Nest(usize, Box<Self>),
    Text(String),
    Cat(Box<Self>, Box<Self>),
    Union(Box<Self>, Box<Self>),
    FlatAlt(Box<Self>, Box<Self>),
    Column(Rc<dyn Fn(usize) -> Self + 'a>),
    Row(Rc<dyn Fn(usize) -> Self + 'a>),
    Nesting(Rc<dyn Fn(usize) -> Self + 'a>),
}


pub fn nil<'a>() -> Doc<'a> {
    Nil
}

pub fn text<'a>(txt: &str) -> Doc<'a> {
    Text(txt.to_string())
}

pub fn space<'a>() -> Doc<'a> {
    text(" ")
}

pub fn line<'a>() -> Doc<'a> {
    FlatAlt(Box::new(Line), Box::new(space()))
}

pub fn hard_line<'a>() -> Doc<'a> {
    Line
}

pub fn column<'a, F>(f: F) -> Doc<'a>
    where
        F: Fn(usize) -> Doc<'a> + 'a
{
    Column(Rc::new(f))
}

pub fn row<'a, F>(f: F) -> Doc<'a>
    where
        F: Fn(usize) -> Doc<'a> + 'a
{
    Row(Rc::new(f))
}

pub fn nesting<'a, F>(f: F) -> Doc<'a>
    where
        F: Fn(usize) -> Doc<'a> + 'a
{
    Nesting(Rc::new(f))
}

impl<'a> Doc<'a> {
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
            FlatAlt(_, y) => *y,
            Union(x, _) => x.flatten(),
            Column(f) => column(move |col| f(col).flatten()),
            Row(f) => row(move |col| f(col).flatten()),
            Nesting(f) => nesting(move |col| f(col).flatten()),
            other => other,
        }
    }

    pub fn pretty(&self, w: usize) -> String {
        let mut linear = SimpleDoc::default();
        let mut stack = vec![(0, self)];
        let arena = Arena::new();
        be(w, 0, 0, &mut stack, &arena, &mut linear);
        format!("{}", linear)
    }
}

fn be<'a, 'b>(
    w: usize,
    mut row: usize,
    mut col: usize,
    stack: &mut Vec<(usize, &'a Doc<'b>)>,
    arena: &'a Arena<Doc<'b>>,
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
                col += txt.len();
                linear.0.push(SimpleDocElem::Text(txt));
            }
            Doc::Cat(x, y) => {
                stack.push((nested, y));
                stack.push((nested, x));
            }
            Doc::Nest(j, x) => {
                stack.push((nested + *j, x));
                col = nested;
            }
            Doc::FlatAlt(x, _) => stack.push((nested, x)),
            Doc::Union(x, y) => {
                let stack_saved = stack.clone();
                let linear_saved = linear.clone();

                stack.push((nested, x));
                be(w, row, col, stack, arena, linear);
                if linear.fits(w - col) {
                    break;
                } else {
                    *stack = stack_saved;
                    *linear = linear_saved;
                    stack.push((nested, y));
                }
            }
            Column(f) => {
                stack.push((nested, arena.alloc(f(col))))
            }
            Row(f) => {
                stack.push((nested, arena.alloc(f(row))))
            }
            Nesting(f) => {
                stack.push((nested, arena.alloc(f(nested))))
            }
        }
    }
}


#[derive(Clone, Debug)]
pub enum SimpleDocElem<'a> {
    Text(&'a str),
    Line(usize),
}

#[derive(Clone, Default)]
pub struct SimpleDoc<'a>(pub Vec<SimpleDocElem<'a>>);

impl SimpleDoc<'_> {
    pub fn fits(&self, mut w: usize) -> bool {
        for elem in &self.0 {
            match elem {
                SimpleDocElem::Text(txt) => {
                    if w < txt.len() {
                        return false;
                    }
                    w -= txt.len();
                }
                SimpleDocElem::Line(_) => break,
            }
        }
        true
    }
}

impl Display for SimpleDoc<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for elem in &self.0 {
            match elem {
                SimpleDocElem::Text(s) => write!(f, "{}", s)?,
                SimpleDocElem::Line(nested) => {
                    let spaces = vec![' '; *nested as usize].into_iter().collect::<String>();
                    write!(f, "\n{}", spaces)?
                }
            }
        }
        Ok(())
    }
}
