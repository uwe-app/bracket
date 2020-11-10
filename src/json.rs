//! Helper functions for working with JSON values.
use serde_json::Value;

static OBJECT: &str = "Object";
static ARRAY: &str = "Array";

pub(crate) fn stringify(value: &Value) -> String {
    match value {
        Value::String(ref s) => s.to_owned(),
        Value::Object(_) => OBJECT.to_owned(),
        Value::Array(ref arr) => format!("{}[{}]", ARRAY, arr.len()),
        _ => value.to_string(),
    }
}

pub(crate) fn unquote(value: &Value) -> String {
    match value {
        Value::String(ref s) => s.to_owned(),
        _ => value.to_string(),
    }
}

// Look up path parts in an object.
pub(crate) fn find_parts<'a, 'b, I>(
    mut it: I,
    doc: &'b Value,
) -> Option<&'b Value>
where
    I: Iterator<Item = &'a str>,
{
    match doc {
        Value::Object(_) | Value::Array(_) => {
            let mut current: Option<&Value> = Some(doc);
            let mut next_part = it.next();
            while let Some(part) = next_part {
                if let Some(target) = current {
                    current = find_field(target, part);
                } else {
                    break;
                }
                next_part = it.next();
                if next_part.is_none() && current.is_some() {
                    return current;
                }
            }
            None
        }
        _ => None,
    }
}

// Look up a field in an array or object.
pub(crate) fn find_field<'b, S: AsRef<str>>(
    target: &'b Value,
    field: S,
) -> Option<&'b Value> {
    match target {
        Value::Object(ref map) => {
            if let Some(val) = map.get(field.as_ref()) {
                return Some(val);
            }
        }
        Value::Array(ref list) => {
            let name = field.as_ref();
            // Support for square-bracket notation, eg: `list.[1]`
            let value = if name.starts_with("[") && name.ends_with("]") {
                &name[1..name.len() - 1]
            } else {
                name
            };
            if let Ok(index) = value.parse::<usize>() {
                return list.get(index);
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
            } else {
                false
            }
        }
        _ => false,
    }
}
