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
            &Predicate::Eq(field, ref value) => {
                println!("field: {}, value: {:?}, props: {:?}", field, value, self.properties);
                &self.properties[field] == value
            },
            &Predicate::Neq(field, ref value) => &self.properties[field] != value,
            &Predicate::Lt(field, ref value) => &self.properties[field] < value,
            &Predicate::Lte(field, ref value) => &self.properties[field] <= value,
            &Predicate::Gt(field, ref value) => &self.properties[field] > value,
            &Predicate::Gte(field, ref value) => &self.properties[field] >= value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        assert!(node.matches(&Predicate::Eq("string", "string".into())), "string == string");
        assert!(!node.matches(&Predicate::Eq("number", "number".into())), "!(number == number)");
    }

    #[test]
    fn neq() {
        let node = create_node();

        assert!(node.matches(&Predicate::Neq("string", "number".into())), "string != number");
        assert!(!node.matches(&Predicate::Neq("number", 42.into())), "!(number != 42)");
    }

    #[test]
    fn lt() {
        let node = create_node();

        assert!(node.matches(&Predicate::Lt("number", 43.into())), "number < 43");
        assert!(!node.matches(&Predicate::Lt("number", 42.into())), "!(number < 42)");
    }

    #[test]
    fn lte() {
        let node = create_node();

        assert!(node.matches(&Predicate::Lte("number", 43.into())), "number <= 43");
        assert!(node.matches(&Predicate::Lte("number", 42.into())), "number <= 42");
        assert!(!node.matches(&Predicate::Lte("number", 41.into())), "!(number <= 41)");
    }

    #[test]
    fn gt() {
        let node = create_node();

        assert!(node.matches(&Predicate::Gt("number", 41.into())), "number > 41");
        assert!(!node.matches(&Predicate::Gt("number", 42.into())), "!(number > 42)");
    }

    #[test]
    fn gte() {
        let node = create_node();

        assert!(node.matches(&Predicate::Gte("number", 41.into())), "number >= 41");
        assert!(node.matches(&Predicate::Gte("number", 42.into())), "number >= 42");
        assert!(!node.matches(&Predicate::Gte("number", 43.into())), "!(number >= 43)");
    }

    #[test]
    fn all() {
        let node = create_node();

        assert!(node.matches(&Predicate::All(&[
            Predicate::Gt("number", 41.into()),
            Predicate::Lt("number", 43.into()),
        ])), "number > 41 && number < 43");

        assert!(!node.matches(&Predicate::All(&[
            Predicate::Gt("number", 41.into()),
            Predicate::Lt("number", 43.into()),
            Predicate::Eq("number", "number".into()),
        ])), "!(number > 41 && number < 43 && number == number)");
    }

    #[test]
    fn any() {
        let node = create_node();

        assert!(node.matches(&Predicate::Any(&[
            Predicate::Gt("number", 41.into()),
            Predicate::Lt("number", 41.into()),
        ])), "number > 41 || number < 41");

        assert!(node.matches(&Predicate::Any(&[
            Predicate::Gt("number", 41.into()),
            Predicate::Lt("number", 43.into()),
        ])), "number > 41 || number < 43");

        assert!(!node.matches(&Predicate::Any(&[
            Predicate::Gt("number", 43.into()),
            Predicate::Lt("number", 41.into()),
        ])), "!(number > 43 || number < 41)");
    }
}
