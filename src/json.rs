use serde_json::Value;

pub fn stringify(value: &Value) -> String {
    match value {
        Value::String(ref s) => s.to_owned(),
        _ => todo!("Stringify other json types"),
    }
}

// Look up path parts in an object.
pub fn find_parts<'a, 'b>(
    parts: Vec<&'a str>,
    doc: &'b Value,
) -> Option<&'b Value> {
    let mut parent = None;
    match doc {
        Value::Object(ref _map) => {
            let mut current: Option<&Value> = Some(doc);
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    if let Some(target) = current {
                        return find_field(&part, target);
                    }
                } else {
                    if let Some(target) = current {
                        parent = find_field(part, target);
                        if parent.is_none() {
                            break;
                        }
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }
        _ => {}
    }
    None
}

// Look up a field in an array or object.
pub fn find_field<'b, S: AsRef<str>>(
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

pub fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Object(ref _map) => return true,
        Value::Array(ref _list) => return true,
        Value::String(ref s) => return s.len() > 0,
        Value::Bool(ref b) => return *b,
        Value::Number(ref n) => {
            if n.is_i64() {
                return n.as_i64().unwrap() != 0;
            } else if n.is_u64() {
                return n.as_u64().unwrap() != 0;
            } else if n.is_f64() {
                return n.as_f64().unwrap() != 0.0;
            }
        }
        _ => {}
    }
    false
}
