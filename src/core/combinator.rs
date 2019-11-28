use crate::core::traits::{best, Doc, SimpleDocElem, TranslationControl, FlattenableDoc, TransState};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct Empty;

impl<'d> Doc<'d> for Empty {
    fn translate<'a>(&'a self, _state: &mut TransState<'a, 'd>) -> TranslationControl {
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

impl<'d> Doc<'d> for Line {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
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

impl<'d> Doc<'d> for Text<'_> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
        state.append(SimpleDocElem::Text(self.txt));
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

impl<'d, D: Doc<'d> + 'd> Doc<'d> for Nest<D> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
        state.push(state.nesting + self.nest, &self.doc);
        state.col = state.nesting;
        TranslationControl::Continue
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

impl<'d, A: Doc<'d> + 'd, B: Doc<'d> + 'd> Doc<'d> for Cat<A, B> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
        state.push(state.nesting, &self.b);
        state.push(state.nesting, &self.a);
        TranslationControl::Continue
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

impl<'d, A: Doc<'d> + 'd, B: Doc<'d> + 'd> Doc<'d> for Union<A, B> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
        let state_saved = state.clone();

        state.push(state_saved.nesting, &self.a);
        best(state);
        if state.fits(state_saved.page_width - state_saved.col) {
            return TranslationControl::Break;
        }

        *state = state_saved;
        state.push(state.nesting, &self.b);
        TranslationControl::Continue
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

impl<'d, A: Doc<'d> + 'd, B> Doc<'d> for FlatAlt<A, B> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
        state.push(state.nesting, &self.a);
        TranslationControl::Continue
    }
}

impl<A, B: FlattenableDoc> FlattenableDoc for FlatAlt<A, B> {
    type Flattened = B::Flattened;

    fn flatten(self) -> Self::Flattened {
        self.b.flatten()
    }
}

pub struct Column<F> {
    f: Rc<F>
}

impl<F> Column<F> {
    pub fn new(f: Rc<F>) -> Self {
        Self {
            f
        }
    }
}

impl<F> Clone for Column<F> {
    fn clone(&self) -> Self {
        Column::new(self.f.clone())
    }
}

impl<'d, D: Doc<'d> + 'd, F: Fn(usize) -> D> Doc<'d> for Column<F> {
    fn translate<'a>(&'a self, state: &mut TransState<'a, 'd>) -> TranslationControl {
        state.push(state.nesting, state.hold((self.f)(state.col)));
        TranslationControl::Continue
    }
}