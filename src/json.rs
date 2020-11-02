use serde_json::{Error, Value};

pub(crate) fn stringify(value: &Value) -> Result<String, Error> {
    match value {
        Value::String(ref s) => Ok(s.to_owned()),
        _ => Ok(value.to_string()),
    }
}

// Look up path parts in an object.
pub(crate) fn find_parts<'a, 'b>(
    parts: Vec<&'a str>,
    doc: &'b Value,
) -> Option<&'b Value> {
    match doc {
        Value::Object(ref _map) => {
            let mut current: Option<&Value> = Some(doc);
            for (i, part) in parts.iter().enumerate() {
                if let Some(target) = current {
                    current = find_field(&part, target);
                    if current.is_none() { break }
                    if i == parts.len() - 1 {
                        return current
                    }
                } else { break }
            }
            None
        }
        _ => None
    }
}

// Look up a field in an array or object.
pub(crate) fn find_field<'b, S: AsRef<str>>(
    field: S,
    parent: &'b Value,
) -> Option<&'b Value> {
    match parent {
        Value::Object(ref map) => {
            if let Some(val) = map.get(field.as_ref()) {
                return Some(val);
            }
        }
        Value::Array(ref list) => {
            if let Ok(index) = field.as_ref().parse::<usize>() {
                if !list.is_empty() && index < list.len() {
                    return Some(&list[index]);
                }
            }
        }
        _ => {}
    }
    None
}

pub(crate) fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Object(_) => true,
        Value::Array(_) => true,
        Value::String(ref s) => s.len() > 0,
        Value::Bool(ref b) => *b,
        Value::Number(ref n) => {
            if n.is_i64() {
                n.as_i64().unwrap() != 0
            } else if n.is_u64() {
                n.as_u64().unwrap() != 0
            } else if n.is_f64() {
                n.as_f64().unwrap() != 0.0
            } else { false }
        }
        _ => false
    }
}
