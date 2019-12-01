use crate::core::combinator::{line, space, Cat, FlatAlt, Line, Nest, Text, Union};
use std::fmt::{Display, Error, Formatter};
use typed_arena::Arena;
use std::rc::Rc;
use std::ops::Try;

#[derive(Copy, Clone, Debug)]
pub enum TranslationControl {
    Break,
    Continue,
}

impl Try for TranslationControl {
    type Ok = ();
    type Error = ();

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            TranslationControl::Break => Err(()),
            TranslationControl::Continue => Ok(()),
        }
    }

    fn from_error(v: Self::Error) -> Self {
        Self::Break
    }

    fn from_ok(v: Self::Ok) -> Self {
        Self::Continue
    }
}

#[derive(Clone)]
pub struct TransState {
    pub page_width: usize,
    pub nesting: usize,
    pub row: usize,
    pub col: usize,
    pub index: usize,

    result: SimpleDoc,
}

impl TransState {
    pub fn append(&mut self, elem: SimpleDocElem) -> &mut Self {
        self.result.0.push(elem);
        self
    }

    pub fn fits(&self, w: usize) -> bool {
        self.result.fits(w)
    }

}



pub trait Doc {
    fn translate(
        &self,
        state: &mut TransState,
    ) -> TranslationControl;

    fn pretty(&self, page_width: usize) -> String {
        let mut state = TransState {
            page_width,
            nesting: 0,
            row: 0,
            col: 0,
            index: 0,
            result: SimpleDoc(vec![]),
        };

        self.translate(&mut state);
        format!("{}", state.result)
    }

    fn cat<D: Doc>(self, rhs: D) -> Cat<Self, D>
    where
        Self: Sized,
    {
        Cat::new(self, rhs)
    }

    fn cat_with_line<D: Doc>(self, rhs: D) -> Cat<Cat<Self, FlatAlt<Line, Text<'static>>>, D>
    where
        Self: Sized,
    {
        self.cat(line()).cat(rhs)
    }

    fn cat_with_space<D: Doc>(self, rhs: D) -> Cat<Cat<Self, Text<'static>>, D>
    where
        Self: Sized,
    {
        self.cat(space()).cat(rhs)
    }

    fn nest(self, nest: usize) -> Nest<Self>
    where
        Self: Sized,
    {
        Nest::new(nest, self)
    }

    fn flat_alt<D: Doc>(self, rhs: D) -> FlatAlt<Self, D>
    where
        Self: Sized
    {
        FlatAlt::new(self, rhs)
    }
}

pub trait FlattenableDoc {
    type Flattened;
    fn flatten(self) -> Self::Flattened;

    fn group(self) -> Union<Self::Flattened, Self> where Self: Clone {
        Union::new(self.clone().flatten(), self)
    }
}

impl<D: Doc + ?Sized> Doc for &D {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        (**self).translate(state)
    }
}

impl<D: Doc + ?Sized> Doc for &mut D {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        (**self).translate(state)
    }
}

impl<D: Doc + ?Sized> Doc for Box<D> {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        (**self).translate(state)
    }
}

impl<D: Doc + ?Sized> Doc for Rc<D> {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        (**self).translate(state)
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
    Line(usize),
}

#[derive(Clone, Default)]
pub struct SimpleDoc(pub Vec<SimpleDocElem>);

impl SimpleDoc {
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
