use std::io;

#[derive(Clone)]
pub enum PathPart<'a> {
    Field(&'a String),
}

pub type Path<'a> = Vec<PathPart<'a>>;

#[derive(Debug)]
pub enum Error {
    EmptyPath,
    FieldIsNotTraversable,
    InvalidValue,
    Io(io::Error),
    MultiInMulti,
    Unknown,
}

pub type KakoiResult<T = ()> = Result<T, Error>;

#[derive(Debug)]
pub enum Selector<'a> {
    AllFields,
    Field(String),
    Multi(Vec<Selector<'a>>),
    Traverse(String, &'a Selector<'a>),
}
