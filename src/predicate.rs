use entities::PrimitiveValue;
use node::Node;

#[derive(Clone, Debug)]
pub enum Predicate<'a> {
    Eq(&'a str, PrimitiveValue),
    Neq(&'a str, PrimitiveValue),
    Lt(&'a str, PrimitiveValue),
    Lte(&'a str, PrimitiveValue),
    Gt(&'a str, PrimitiveValue),
    Gte(&'a str, PrimitiveValue),
    All(&'a [Predicate<'a>]),
    Any(&'a [Predicate<'a>]),
}

pub trait MatchesPredicate {
    fn matches(&self, predicate: &Predicate) -> bool;
}

impl MatchesPredicate for Node {
    fn matches(&self, predicate: &Predicate) -> bool {
        match predicate {
            &Predicate::All(predicates) => predicates.iter().all(|p| self.matches(p)),
            &Predicate::Any(predicates) => predicates.iter().any(|p| self.matches(p)),
            &Predicate::Eq(field, ref value) => &self.properties[field] == value,
            &Predicate::Neq(field, ref value) => &self.properties[field] != value,
            &Predicate::Lt(field, ref value) => &self.properties[field] < value,
            &Predicate::Lte(field, ref value) => &self.properties[field] <= value,
            &Predicate::Gt(field, ref value) => &self.properties[field] > value,
            &Predicate::Gte(field, ref value) => &self.properties[field] >= value,
        }
    }
}
