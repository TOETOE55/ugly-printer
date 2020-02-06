use crate::core::combinator::{line, space, Cat, FlatAlt, Line, Nest, Text, Union};
use std::fmt::{Display, Error, Formatter};
use std::rc::Rc;

#[derive(Clone, Copy, Default)]
pub struct PrettyState {
    pub page_width: i64,
    pub placed: i64,
    pub row: i64,
    pub col: i64,
    pub indent: i64,
    pub index: i64,
}

impl PrettyState {
    pub fn with_page_width(self, page_width: i64) -> Self {
        Self { page_width, ..self }
    }

    pub fn with_placed(self, placed: i64) -> Self {
        Self { placed, ..self }
    }

    pub fn with_row(self, row: i64) -> Self {
        Self { row, ..self }
    }

    pub fn with_col(self, col: i64) -> Self {
        Self { col, ..self }
    }

    pub fn with_indent(self, indent: i64) -> Self {
        Self { indent, ..self }
    }

    pub fn with_index(self, index: i64) -> Self {
        Self { index, ..self }
    }
}

pub trait Doc {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    );

    fn pretty(&self, page_width: i64) -> String {
        let mut simple_doc = SimpleDoc(vec![]);
        let state = PrettyState::default().with_page_width(page_width);
        self.best(state, &mut simple_doc, &mut |_, _| {});
        format!("{}", simple_doc)
    }

    fn cat<D: Doc>(self, rhs: D) -> Cat<Self, D>
    where
        Self: Sized,
    {
        Cat::new(self, rhs)
    }

    fn cat_with_line<D: Doc>(self, rhs: D) -> Cat<Cat<Self, FlatAlt<Line, Text>>, D>
    where
        Self: Sized,
    {
        self.cat(line()).cat(rhs)
    }

    fn cat_with_space<D: Doc>(self, rhs: D) -> Cat<Cat<Self, Text>, D>
    where
        Self: Sized,
    {
        self.cat(space()).cat(rhs)
    }

    fn nest(self, nest: i64) -> Nest<Self>
    where
        Self: Sized,
    {
        Nest::new(nest, self)
    }

    fn flat_alt<D: Doc>(self, rhs: D) -> FlatAlt<Self, D>
    where
        Self: Sized,
    {
        FlatAlt::new(self, rhs)
    }
}

pub trait FlattenableDoc: Doc + Clone {
    type Flattened: FlattenableDoc;
    fn flatten(self) -> Self::Flattened;

    fn group(self) -> Union<Self::Flattened, Self> {
        Union::new(self.clone().flatten(), self)
    }
}

impl<D: Doc + ?Sized> Doc for &D {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        (**self).best(pretty_state, ret, cont)
    }
}

impl<D: Doc + ?Sized> Doc for &mut D {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        (**self).best(pretty_state, ret, cont)
    }
}

impl<D: Doc + ?Sized> Doc for Box<D> {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        (**self).best(pretty_state, ret, cont)
    }
}

impl<D: Doc + ?Sized> Doc for Rc<D> {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        (**self).best(pretty_state, ret, cont)
    }
}

impl<D: FlattenableDoc> FlattenableDoc for Box<D> {
    type Flattened = D::Flattened;

    fn flatten(self) -> Self::Flattened {
        (*self).flatten()
    }
}

#[derive(Clone, Debug)]
pub enum SimpleDocElem {
    Text(String),
    Line(i64),
}

#[derive(Clone, Default)]
pub struct SimpleDoc(Vec<SimpleDocElem>);

impl SimpleDoc {
    pub fn fits(&self, mut w: i64) -> bool {
        for elem in &self.0 {
            if w < 0 {
                return false;
            }
            match elem {
                SimpleDocElem::Text(txt) => {
                    w -= txt.len() as i64;
                }
                SimpleDocElem::Line(_) => break,
            }
        }
        true
    }

    pub fn add(&mut self, elem: SimpleDocElem) {
        self.0.push(elem)
    }

    pub fn append(&mut self, mut other: Self) {
        self.0.append(&mut other.0)
    }
}

impl Display for SimpleDoc {
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

impl IntoIterator for SimpleDoc {
    type Item = SimpleDocElem;
    type IntoIter = std::vec::IntoIter<SimpleDocElem>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
