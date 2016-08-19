use entities::PrimitiveValue;
use node::Node;
use self::Predicate::*;

#[derive(Clone, Debug, PartialEq)]
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

impl<'a> Predicate<'a> {
    pub fn get_fields(&self) -> Vec<&str> {
        match self {
            &Eq(field, _) | &Neq(field, _) |
            &Lt(field, _) | &Lte(field, _) |
            &Gt(field, _) | &Gte(field, _) =>
                vec![field],
            &All(predicates) | &Any(predicates) =>
                predicates.iter().flat_map(Predicate::get_fields).collect(),
        }
    }
}

pub trait MatchesPredicate {
    fn matches(&self, predicate: &Predicate) -> bool;
}

impl MatchesPredicate for Node {
    fn matches(&self, predicate: &Predicate) -> bool {
        match predicate {
            &All(predicates) => predicates.iter().all(|p| self.matches(p)),
            &Any(predicates) => predicates.iter().any(|p| self.matches(p)),
            &Eq(field, ref value) => &self.properties[field] == value,
            &Neq(field, ref value) => &self.properties[field] != value,
            &Lt(field, ref value) => &self.properties[field] < value,
            &Lte(field, ref value) => &self.properties[field] <= value,
            &Gt(field, ref value) => &self.properties[field] > value,
            &Gte(field, ref value) => &self.properties[field] >= value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Predicate::*;
    use std::collections::HashMap;
    use node::Node;
    use value::Value;

    fn create_node() -> Node {
        let mut node = Node {
            id: "id".to_string(),
            properties: HashMap::new()
        };
        node.properties.insert("string".to_string(), "Sstring".into());
        node.properties.insert("number".to_string(), Value::I64(42));

        node
    }

    #[test]
    fn eq() {
        let node = create_node();

        assert!(node.matches(&Eq("string", "string".into())), "string == string");
        assert!(!node.matches(&Eq("number", "number".into())), "!(number == number)");
    }

    #[test]
    fn neq() {
        let node = create_node();

        assert!(node.matches(&Neq("string", "number".into())), "string != number");
        assert!(!node.matches(&Neq("number", 42.into())), "!(number != 42)");
    }

    #[test]
    fn lt() {
        let node = create_node();

        assert!(node.matches(&Lt("number", 43.into())), "number < 43");
        assert!(!node.matches(&Lt("number", 42.into())), "!(number < 42)");
    }

    #[test]
    fn lte() {
        let node = create_node();

        assert!(node.matches(&Lte("number", 43.into())), "number <= 43");
        assert!(node.matches(&Lte("number", 42.into())), "number <= 42");
        assert!(!node.matches(&Lte("number", 41.into())), "!(number <= 41)");
    }

    #[test]
    fn gt() {
        let node = create_node();

        assert!(node.matches(&Gt("number", 41.into())), "number > 41");
        assert!(!node.matches(&Gt("number", 42.into())), "!(number > 42)");
    }

    #[test]
    fn gte() {
        let node = create_node();

        assert!(node.matches(&Gte("number", 41.into())), "number >= 41");
        assert!(node.matches(&Gte("number", 42.into())), "number >= 42");
        assert!(!node.matches(&Gte("number", 43.into())), "!(number >= 43)");
    }

    #[test]
    fn all() {
        let node = create_node();

        assert!(node.matches(&All(&[
            Gt("number", 41.into()),
            Lt("number", 43.into()),
        ])), "number > 41 && number < 43");

        assert!(!node.matches(&All(&[
            Gt("number", 41.into()),
            Lt("number", 43.into()),
            Eq("number", "number".into()),
        ])), "!(number > 41 && number < 43 && number == number)");
    }

    #[test]
    fn any() {
        let node = create_node();

        assert!(node.matches(&Any(&[
            Gt("number", 41.into()),
            Lt("number", 41.into()),
        ])), "number > 41 || number < 41");

        assert!(node.matches(&Any(&[
            Gt("number", 41.into()),
            Lt("number", 43.into()),
        ])), "number > 41 || number < 43");

        assert!(!node.matches(&Any(&[
            Gt("number", 43.into()),
            Lt("number", 41.into()),
        ])), "!(number > 43 || number < 41)");
    }
}
