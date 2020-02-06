use crate::core::traits::{Doc, FlattenableDoc, PrettyState, SimpleDoc, SimpleDocElem};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub struct Empty;

impl Doc for Empty {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        cont(pretty_state, ret)
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
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        ret.add(SimpleDocElem::Line(pretty_state.indent));
        let new_state = pretty_state
            .with_row(pretty_state.row + 1)
            .with_placed(pretty_state.indent)
            .with_index(pretty_state.index + 1);
        cont(new_state, ret);
    }
}

impl FlattenableDoc for Line {
    type Flattened = Self;

    fn flatten(self) -> Self::Flattened {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    txt: String,
}

impl Doc for Text {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        ret.add(SimpleDocElem::Text(self.txt.clone()));
        let new_state = pretty_state
            .with_col(pretty_state.col + self.txt.len() as i64)
            .with_placed(pretty_state.placed + self.txt.len() as i64)
            .with_index(pretty_state.index + self.txt.len() as i64);
        cont(new_state, ret)
    }
}

impl FlattenableDoc for Text {
    type Flattened = Self;

    fn flatten(self) -> Self::Flattened {
        self
    }
}

pub fn text(txt: &str) -> Text {
    Text {
        txt: txt.to_string(),
    }
}

pub fn space() -> Text {
    text(" ")
}

pub fn line() -> FlatAlt<Line, Text> {
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
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        let new_state = pretty_state
            .with_col(pretty_state.placed)
            .with_indent(pretty_state.indent + self.nest);
        self.doc.best(new_state, ret, &mut |state, ret| {
            cont(state.with_indent(pretty_state.indent), ret)
        });
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
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        self.a.best(pretty_state, ret, &mut |pretty_state, ret| {
            self.b.best(pretty_state, ret, cont)
        });
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
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        let mut x_sd = SimpleDoc::default();
        self.a.best(pretty_state, &mut x_sd, cont);
        if x_sd.fits(pretty_state.page_width - pretty_state.placed) {
            ret.append(x_sd);
        } else {
            self.b.best(pretty_state, ret, cont);
        }
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

impl<A: Doc, B: Clone> Doc for FlatAlt<A, B> {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        self.a.best(pretty_state, ret, cont);
    }
}

impl<A: Doc + Clone, B: FlattenableDoc> FlattenableDoc for FlatAlt<A, B> {
    type Flattened = B::Flattened;

    fn flatten(self) -> Self::Flattened {
        self.b.flatten()
    }
}

pub struct Column<'a, D> {
    f: Rc<dyn Fn(i64) -> D + 'a>,
}

impl<'a, D> Column<'a, D> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(i64) -> D + 'a,
    {
        Self { f: Rc::new(f) }
    }
}

impl<D> Clone for Column<'_, D> {
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}

impl<D: Doc> Doc for Column<'_, D> {
    fn best(
        &self,
        pretty_state: PrettyState,
        ret: &mut SimpleDoc,
        cont: &mut dyn FnMut(PrettyState, &mut SimpleDoc),
    ) {
        (self.f)(pretty_state.col).best(pretty_state, ret, cont)
    }
}

impl<'a, D: 'a> FlattenableDoc for Column<'a, D>
where
    D: FlattenableDoc,
{
    type Flattened = Column<'a, D::Flattened>;

    fn flatten(self) -> Self::Flattened {
        Column::new(move |col| (self.f)(col).flatten())
    }
}

pub fn column<'a, D: Doc, F: 'a>(f: F) -> Column<'a, D>
where
    F: Fn(i64) -> D,
{
    Column::new(f)
}
