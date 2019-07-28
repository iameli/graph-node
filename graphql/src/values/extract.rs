use graphql_parser::query as q;

trait Extract {
    fn get(&self, key: &str) -> Option<&q::Value>;
    fn values(&self) -> Option<&Vec<q::Value>>;
}

impl Extract for q::Value {
    fn get(&self, key: &str) -> Option<&q::Value> {
        match self {
            q::Value::Object(map) => map.get(&String::from(key)),
            _ => None,
        }
    }

    fn values(&self) -> Option<&Vec<q::Value>> {
        match self {
            q::Value::List(values) => Some(values),
            _ => None,
        }
    }
}
