
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

#[derive(Clone)]
pub struct TransState<'a, 'd> {
    pub page_width: usize,
    pub row: usize,
    pub col: usize,
    pub index: usize,

    stack: Vec<(usize, &'a Doc<'d>)>,
    result: SimpleDoc<'a>,
    holder: &'a Arena<Doc<'d>>,
}

impl<'a, 'd> TransState<'a, 'd> {
    pub fn pop(&mut self) -> Option<(usize, &'a Doc<'d>)> {
        self.stack.pop()
    }

    pub fn push(&mut self, nested: usize, doc: &'a Doc<'d>) -> &mut Self {
        self.stack.push((nested, doc));
        self
    }

    pub fn append(&mut self, elem: SimpleDocElem<'a>) -> &mut Self {
        self.result.0.push(elem);
        self
    }

    pub fn fits(&self, w: usize) -> bool {
        self.result.fits(w)
    }

    pub fn hold(&self, doc: Doc<'d>) -> &'a mut Doc<'d> {
        self.holder.alloc(doc)
    }
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
        let mut state = TransState {
            page_width: w,
            row: 0,
            col: 0,
            index: 0,
            stack: vec![(0, self)],
            result: Default::default(),
            holder: &Default::default()
        };
        be(&mut state);
        format!("{}", state.result)
    }
}

fn be(state: &mut TransState) {
    while let Some((nested, doc)) = state.pop() {
        match doc {
            Doc::Nil => {}
            Doc::Line => {
                state.append(SimpleDocElem::Line(nested));
                state.row += 1;
            }
            Doc::Text(txt) => {
                state.col += txt.len();
                state.append(SimpleDocElem::Text(txt));
            }
            Doc::Cat(x, y) => {
                state.push(nested, y)
                    .push(nested, x);
            }
            Doc::Nest(j, x) => {
                state.push(nested + *j, x);
                state.col = nested;
            }
            Doc::FlatAlt(x, _) => {
                state.push(nested, x);
            },
            Doc::Union(x, y) => {
                let state_save = state.clone();

                state.push(nested, x);
                be(state);
                if state.fits(state_save.page_width - state_save.col) {
                    break;
                }

                *state = state_save;
                state.push(nested, y);
            }
            Column(f) => {
                state.push(nested, state.hold(f(state.col)));
            }
            Row(f) => {
                state.push(nested, state.hold(f(state.row)));
            }
            Nesting(f) => {
                state.push(nested, state.hold(f(nested)));
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
