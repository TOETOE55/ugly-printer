use crate::core::combinator::{line, space, Cat, FlatAlt, Line, Nest, Text, Union};
use std::fmt::{Display, Error, Formatter};
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

    fn from_error(_: Self::Error) -> Self {
        Self::Break
    }

    fn from_ok(_: Self::Ok) -> Self {
        Self::Continue
    }
}

#[derive(Clone)]
pub struct TransState<'a> {
    pub page_width: i64,
    pub nesting: i64,
    pub row: i64,
    pub col: i64,
    pub index: i64,
    pub stack: Vec<(i64, &'a dyn Doc)>
}




pub trait Doc {
    fn translate<'a>(
        &'a self,
        state: &mut TransState<'a>,
        result: &mut SimpleDoc<'a>,
    ) -> TranslationControl;

    fn pretty<'a>(&'a self, page_width: i64) -> String where Self: Sized {
        let mut state = TransState {
            page_width,
            nesting: 0,
            row: 0,
            col: 0,
            index: 0,
            stack: vec![(0, self as &'a dyn Doc)]
        };

        let mut simple_doc = SimpleDoc(vec![]);
        self.translate(&mut state, &mut simple_doc);
        format!("{}", simple_doc)
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

    fn nest(self, nest: i64) -> Nest<Self>
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

pub trait FlattenableDoc: Doc + Clone {
    type Flattened: FlattenableDoc;
    fn flatten(self) -> Self::Flattened;

    fn group(self) -> Union<Self::Flattened, Self> {
        Union::new(self.clone().flatten(), self)
    }
}

impl<D: Doc + ?Sized> Doc for &D {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        (**self).translate(state, result)
    }
}

impl<D: Doc + ?Sized> Doc for &mut D {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        (**self).translate(state, result)
    }
}

impl<D: Doc + ?Sized> Doc for Box<D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl{
        (**self).translate(state, result)
    }
}

impl<D: Doc + ?Sized> Doc for Rc<D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        (**self).translate(state, result)
    }
}

impl<D: FlattenableDoc> FlattenableDoc for Box<D> {
    type Flattened = D::Flattened;

    fn flatten(self) -> Self::Flattened {
        (*self).flatten()
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
