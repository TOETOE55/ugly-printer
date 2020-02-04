
use Doc::*;
use std::fmt::{Display, Formatter, Error};

#[derive(Clone)]
pub enum Doc {
    Nil,
    Line,
    Nest(i64, Box<Self>),
    Text(String),
    Cat(Box<Self>, Box<Self>),
    Union(Box<Self>, Box<Self>),
    FlatAlt(Box<Self>, Box<Self>),
}

impl Doc {
    pub fn nest(self, nested: i64) -> Self {
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
            other => other,
        }
    }

    pub fn pretty(&self, w: i64) -> String {
        let mut pretty_state = PrettyState {
            page_width: w,
            placed: 0,
            stack: vec![(0, self)]
        };
        let mut simple = SimpleDoc(vec![]);
        be(&mut pretty_state, &mut simple);
        format!("{}", simple)
    }
}

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
    FlatAlt(Box::new(Line), Box::new(space()))
}

pub fn soft_line() -> Doc {
    line().group()
}

pub fn hard_line() -> Doc {
    Line
}

pub fn line_break() -> Doc {
    FlatAlt(Box::new(Line), Box::new(nil()))
}

pub fn soft_line_break() -> Doc {
    line_break().group()
}

#[derive(Clone)]
struct PrettyState<'a> {
    page_width: i64,
    placed: i64,
    stack: Vec<(i64, &'a Doc)>,
}

fn be<'a>(pretty_state: &mut PrettyState<'a>, result: &mut SimpleDoc<'a>) {
    while let Some((indent, doc)) = pretty_state.stack.pop() {
        match doc {
            Nil => {},
            Line => {
                result.0.push(SimpleDocElem::Line(indent));
                pretty_state.placed = indent;
            },
            Nest(j, x) => {
                pretty_state.stack.push((indent + *j, x));
            },
            Text(txt) => {
                result.0.push(SimpleDocElem::Text(txt));
                pretty_state.placed += txt.len() as i64;
            },
            Cat(x, y) => {
                pretty_state.stack.push((indent, y));
                pretty_state.stack.push((indent, x));
            },
            Union(x, y) => {
                let mut pretty_state_clone = pretty_state.clone();

                pretty_state_clone.stack.push((indent, x));
                let mut x_sd = SimpleDoc(vec![]);
                be(&mut pretty_state_clone, &mut x_sd);
                if x_sd.fits(pretty_state.page_width - pretty_state.placed) {
                    result.0.append(&mut x_sd.0);
                    break;
                }

                pretty_state.stack.push((indent, y));
            },
            FlatAlt(x, _) => {
                pretty_state.stack.push((indent, x));
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum SimpleDocElem<'a> {
    Text(&'a str),
    Line(i64),
}

#[derive(Clone, Default)]
pub struct SimpleDoc<'a>(pub Vec<SimpleDocElem<'a>>);

impl SimpleDoc<'_> {
    pub fn fits(&self, mut w: i64) -> bool {
        for elem in &self.0 {
            if w < 0 { return false; }
            match elem {
                SimpleDocElem::Text(txt) => {
                    w -= txt.len() as i64;
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
