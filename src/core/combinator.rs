use crate::core::traits::{Doc, SimpleDocElem, TranslationControl, FlattenableDoc, TransState};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct Empty;

impl Doc for Empty {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
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
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        state.append(SimpleDocElem::Line(state.nesting));
        state.row += 1;
        state.index += 1;
        TranslationControl::Continue
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Text<'a> {
    txt: &'a str,
}

impl Doc for Text<'_> {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        state.append(SimpleDocElem::Text(self.txt.to_string()));
        state.col += self.txt.len();
        state.index += self.txt.len();
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
    nest: usize,
    doc: D,
}

impl<D> Nest<D> {
    pub fn new(nest: usize, doc: D) -> Self {
        Self { nest, doc }
    }
}

impl<D: Doc> Doc for Nest<D> {
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        state.col = state.nesting;
        state.nesting += self.nest;
        self.doc.translate(state)
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
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        self.a.translate(state)?;
        self.b.translate(state)
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
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        let state_saved = state.clone();

        self.a.translate(state)?;
        if state.fits(state_saved.page_width - state_saved.col) {
            return TranslationControl::Break;
        }

        *state = state_saved;
        self.b.translate(state)
    }
}

impl<A: FlattenableDoc, B> FlattenableDoc for Union<A, B> {
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
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        self.a.translate(state)
    }
}

impl<A, B: FlattenableDoc> FlattenableDoc for FlatAlt<A, B> {
    type Flattened = B::Flattened;

    fn flatten(self) -> Self::Flattened {
        self.b.flatten()
    }
}

pub struct Column<'a, D> {
    f: Rc<dyn Fn(usize) -> D + 'a>
}

impl<'a, D> Column<'a, D> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(usize) -> D + 'a
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
    fn translate(&self, state: &mut TransState) -> TranslationControl {
        (self.f)(state.col).translate(state)
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
    F: Fn(usize) -> D
{
    Column::new(f)
}