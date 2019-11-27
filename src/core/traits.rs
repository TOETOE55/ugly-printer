use std::fmt::{Display, Error, Formatter};

pub fn pretty<D: Doc>(doc: &D, w: usize) -> String {
    let mut linear = SimpleDoc(vec![]);
    let mut stack = vec![(0, doc as &dyn Doc)];
    best(w, 0, 0, &mut stack, &mut linear);
    format!("{}", linear)
}

pub trait Doc {
    fn compile<'a>(
        &'a self,
        w: usize,
        nested: usize,
        row: &mut usize,
        col: &mut usize,
        stack: &mut Vec<(usize, &'a dyn Doc)>,
        linear: &mut SimpleDoc<'a>,
    );
}

fn best<'a>(
    w: usize,
    mut row: usize,
    mut col: usize,
    stack: &mut Vec<(usize, &'a dyn Doc)>,
    linear: &mut SimpleDoc<'a>,
) {
    while let Some((nested, doc)) = stack.pop() {
        doc.compile(w, nested, &mut row, &mut col, stack, linear)
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



