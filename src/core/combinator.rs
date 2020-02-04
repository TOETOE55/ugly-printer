use crate::core::traits::{Doc, SimpleDocElem, TranslationControl, FlattenableDoc, TransState, SimpleDoc};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct Empty;

impl Doc for Empty {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        TranslationControl::Continue
    }
}

impl FlattenableDoc for Empty {
    type Flattened = Self;

    fn flatten(self) -> Self::Flattened {
        self
    }
}

pub fn empty() -> Empty {
    Empty
}

#[derive(Copy, Clone, Debug)]
pub struct Line;

impl Doc for Line {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        result.0.push(SimpleDocElem::Line(state.nesting));
        state.row += 1;
        state.index += 1;
        TranslationControl::Continue
    }
}

impl FlattenableDoc for Line {
    type Flattened = Self;

    fn flatten(self) -> Self::Flattened {
        self
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Text<'a> {
    txt: &'a str,
}

impl Doc for Text<'_> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        result.0.push(SimpleDocElem::Text(self.txt));
        state.col += self.txt.len() as i64;
        state.index += self.txt.len() as i64;
        TranslationControl::Continue
    }
}

impl FlattenableDoc for Text<'_> {
    type Flattened = Self;

    fn flatten(self) -> Self::Flattened {
        self
    }
}

pub fn text(txt: &str) -> Text {
    Text { txt }
}

pub fn space() -> Text<'static> {
    text(" ")
}

pub fn line() -> FlatAlt<Line, Text<'static>> {
    FlatAlt::new(Line, space())
}

pub fn hard_line() -> Line {
    Line
}

#[derive(Copy, Clone, Debug)]
pub struct Nest<D> {
    nest: i64,
    doc: D,
}

impl<D> Nest<D> {
    pub fn new(nest: i64, doc: D) -> Self {
        Self { nest, doc }
    }
}

impl<D: Doc> Doc for Nest<D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        state.col = state.nesting;
        state.nesting += self.nest;
        self.doc.translate(state, result)
    }
}

impl<D: FlattenableDoc> FlattenableDoc for Nest<D> {
    type Flattened = Nest<D::Flattened>;

    fn flatten(self) -> Self::Flattened {
        Nest::new(self.nest, self.doc.flatten())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Cat<A, B> {
    a: A,
    b: B,
}

impl<A, B> Cat<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Doc, B: Doc> Doc for Cat<A, B> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        self.a.translate(state, result)?;
        self.b.translate(state, result)
    }
}

impl<A: FlattenableDoc, B: FlattenableDoc> FlattenableDoc for Cat<A, B> {
    type Flattened = Cat<A::Flattened, B::Flattened>;

    fn flatten(self) -> Self::Flattened {
        Cat::new(self.a.flatten(), self.b.flatten())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Union<A, B> {
    a: A,
    b: B,
}

impl<A, B> Union<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Doc, B: Doc> Doc for Union<A, B> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        let mut state_saved = state.clone();

        self.a.translate(&mut state_saved, result)?;
        if result.fits(state.page_width - state.col) {
            return TranslationControl::Break;
        }

        self.b.translate(state, result)
    }
}

impl<A: FlattenableDoc, B: Doc + Clone> FlattenableDoc for Union<A, B> {
    type Flattened = A::Flattened;

    fn flatten(self) -> Self::Flattened {
        self.a.flatten()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FlatAlt<A, B> {
    a: A,
    b: B,
}

impl<A, B> FlatAlt<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Doc, B> Doc for FlatAlt<A, B> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        self.a.translate(state, result)
    }
}

impl<A: Doc + Clone, B: FlattenableDoc> FlattenableDoc for FlatAlt<A, B> {
    type Flattened = B::Flattened;

    fn flatten(self) -> Self::Flattened {
        self.b.flatten()
    }
}
/*
pub struct Column<'a, D> {
    f: Rc<dyn Fn(i64) -> D + 'a>
}

impl<'a, D> Column<'a, D> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(i64) -> D + 'a
    {
        Self {
            f: Rc::new(f)
        }
    }
}

impl<D> Clone for Column<'_ ,D> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone()
        }
    }
}

impl<D: Doc> Doc for Column<'_, D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a>, result: &mut SimpleDoc<'a>) -> TranslationControl {
        (self.f)(state.col).translate(state,result)
    }
}

impl<'a, D: 'a> FlattenableDoc for Column<'a, D>
where
    D: FlattenableDoc
{
    type Flattened = Column<'a, D::Flattened>;

    fn flatten(self) -> Self::Flattened {
        Column::new(move |col| (self.f)(col).flatten())
    }
}

pub fn column<'a, D: Doc, F: 'a>(f: F) -> Column<'a, D>
where
    F: Fn(i64) -> D
{
    Column::new(f)
}
*/