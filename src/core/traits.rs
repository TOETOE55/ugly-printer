use crate::core::combinator::{line, space, Cat, FlatAlt, Line, Nest, Text, Union};
use std::fmt::{Display, Error, Formatter};
use typed_arena::Arena;
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum TranslationControl {
    Break,
    Continue,
}

#[derive(Clone)]
pub struct TransState<'a, 'd> {
    pub page_width: usize,
    pub nesting: usize,
    pub row: usize,
    pub col: usize,
    pub index: usize,

    stack: Vec<(usize, &'a (dyn Doc<'d> + 'd))>,
    result: SimpleDoc<'a>,
}

impl<'a, 'd> TransState<'a, 'd> {
    pub fn pop(&mut self) -> Option<(usize, &'a (dyn Doc<'d> + 'd))> {
        self.stack.pop()
    }

    pub fn push(&mut self, nested: usize, doc: &'a (dyn Doc<'d> + 'd)) -> &mut Self {
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
}

#[derive(Copy, Clone)]
pub struct DocHolder<'a, 'd> {
    alloc: &'a Arena<Box<dyn Doc<'d> + 'd>>
}

impl<'a, 'd> DocHolder<'a, 'd> {
    pub fn hold<D: Doc<'d> + 'd>(self, doc: D) -> &'a mut (dyn Doc<'d> + 'd) {
        self.alloc.alloc(Box::new(doc))
    }
}

pub trait Doc<'d> {
    fn translate<'a>(
        &'a self,
        state: &mut TransState<'a, 'd>,
        holder: DocHolder<'a, 'd>
    ) -> TranslationControl;

    fn cat<D: Doc<'d>>(self, rhs: D) -> Cat<Self, D>
    where
        Self: Sized,
    {
        Cat::new(self, rhs)
    }

    fn cat_with_line<D: Doc<'d> + 'd>(self, rhs: D) -> Cat<Cat<Self, FlatAlt<Line, Text<'static>>>, D>
    where
        Self: Sized + 'd,
    {
        self.cat(line()).cat(rhs)
    }

    fn cat_with_space<D: Doc<'d>>(self, rhs: D) -> Cat<Cat<Self, Text<'static>>, D>
    where
        Self: Sized + 'd,
    {
        self.cat(space()).cat(rhs)
    }

    fn nest(self, nest: usize) -> Nest<Self>
    where
        Self: Sized,
    {
        Nest::new(nest, self)
    }

    fn flat_alt<D: Doc<'d>>(self, rhs: D) -> FlatAlt<Self, D>
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

impl<'d, D: Doc<'d> + ?Sized> Doc<'d> for &D {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>, holder: DocHolder<'a, 'd>) -> TranslationControl {
        (**self).translate(state, holder)
    }
}

impl<'d, D: Doc<'d> + ?Sized> Doc<'d> for &mut D {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>, holder: DocHolder<'a, 'd>) -> TranslationControl {
        (**self).translate(state, holder)
    }
}

impl<'d, D: Doc<'d> + ?Sized> Doc<'d> for Box<D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>, holder: DocHolder<'a, 'd>) -> TranslationControl {
        (**self).translate(state, holder)
    }
}

impl<'d, D: Doc<'d> + ?Sized> Doc<'d> for Rc<D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>, holder: DocHolder<'a, 'd>) -> TranslationControl {
        (**self).translate(state, holder)
    }
}

impl<D: FlattenableDoc> FlattenableDoc for Box<D> {
    type Flattened = D::Flattened;

    fn flatten(self) -> Self::Flattened {
        (*self).flatten()
    }
}


pub fn pretty<'d, D: Doc<'d> + 'd>(doc: &D, page_width: usize) -> String {
    let mut state = TransState {
        page_width,
        nesting: 0,
        row: 0,
        col: 0,
        index: 0,
        stack: vec![(0, doc as &(dyn Doc))],
        result: SimpleDoc(vec![])
    };

    let arena = Arena::new();
    let holder = DocHolder {
        alloc: &arena
    };
    best(&mut state, holder);
    format!("{}", state.result)
}

pub fn best<'d, 't>(state: &mut TransState<'t, 'd>, holder: DocHolder<'t, 'd>) {
    while let Some((nested, doc)) = state.pop() {
        state.nesting = nested;
        if let TranslationControl::Break =
            doc.translate(state, holder)
        {
            break;
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
