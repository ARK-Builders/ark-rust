use serde_json::{json, map::Entry, Map, Value};

pub fn merge(origin: Value, new_data: Value) -> Value {
    match (origin, new_data) {
        (Value::Object(old), Value::Object(new)) => merge_object(old, new),
        (Value::Array(old), Value::Array(new)) => merge_vec(old, new),
        (Value::Array(mut old), new) => {
            if !old.is_empty()
                && std::mem::discriminant(&old[0])
                    == std::mem::discriminant(&new)
            {
                old.push(new);
                Value::Array(old)
            } else if old.is_empty() {
                json!([new])
            } else {
                Value::Array(old)
            }
        }
        (old, Value::Array(mut new_data)) => {
            if !new_data.is_empty()
                && std::mem::discriminant(&old)
                    == std::mem::discriminant(&new_data[0])
            {
                new_data.insert(0, old);
                Value::Array(new_data)
            } else {
                // Different types, keep old data
                old
            }
        }
        (old, Value::Null) => old,
        (Value::Null, new) => new,
        (old, new) => {
            if std::mem::discriminant(&old) == std::mem::discriminant(&new)
                && old != new
            {
                json!([old, new])
            } else {
                // different types keep old data
                old
            }
        }
    }
}

fn merge_object(
    mut origin: Map<String, Value>,
    new_data: Map<String, Value>,
) -> Value {
    for (key, value) in new_data.into_iter() {
        match origin.entry(&key) {
            Entry::Vacant(e) => {
                e.insert(value);
            }
            Entry::Occupied(prev) => {
                // Extract entry to manipulate it
                let prev = prev.remove();
                match (prev, value) {
                    (Value::Array(old_data), Value::Array(new_data)) => {
                        let updated = merge_vec(old_data, new_data);
                        origin.insert(key, updated);
                    }
                    (Value::Array(d), Value::Null) => {
                        origin.insert(key, Value::Array(d));
                    }
                    (Value::Array(mut old_data), new_data) => {
                        if old_data.iter().all(|val| {
                            std::mem::discriminant(&new_data)
                                == std::mem::discriminant(val)
                        }) {
                            old_data.push(new_data);
                        }
                        origin.insert(key, json!(old_data));
                    }
                    (old, Value::Array(mut new_data)) => {
                        if new_data.iter().all(|val| {
                            std::mem::discriminant(&old)
                                == std::mem::discriminant(val)
                        }) {
                            new_data.insert(0, old);
                            origin.insert(key, json!(new_data));
                        } else {
                            // Different types, just keep old data
                            origin.insert(key, old);
                        }
                    }
                    (old, new) => {
                        // Only create array if same type
                        if std::mem::discriminant(&old)
                            == std::mem::discriminant(&new)
                            && old != new
                        {
                            origin.insert(key, json!([old, new]));
                        } else {
                            // Keep old value
                            origin.insert(key, old);
                        }
                    }
                }
            }
        }
    }
    Value::Object(origin)
}

fn merge_vec(original: Vec<Value>, new_data: Vec<Value>) -> Value {
    if original.is_empty() {
        Value::Array(new_data)
    } else if new_data.is_empty() {
        Value::Array(original)
    } else {
        // Check that values are the same type. Return array of type original[0]
        let discriminant = std::mem::discriminant(&original[0]);
        let mut filtered: Vec<_> = original
            .into_iter()
            .filter(|v| std::mem::discriminant(v) == discriminant)
            .collect();
        let new: Vec<_> = new_data
            .into_iter()
            .filter(|v| {
                std::mem::discriminant(v) == discriminant
                    && filtered.iter().all(|val| val != v)
            })
            .collect();
        filtered.extend(new);
        Value::Array(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(json ! ("old"), json ! ("new"), json ! (["old", "new"]))]
    #[case(json ! (["old1", "old2"]), json ! ("new"), json ! (["old1", "old2", "new"]))]
    #[case(json ! ("same"), json ! ("same"), json ! ("same"))]
    #[case(json ! ({
    "a": ["An array"],
    "b": 1,
    }), json ! ({"c": "A string"}), json ! ({"a": ["An array"], "b": 1, "c": "A string"}))]
    #[case(json ! ({"a": "Object"}), json ! ("A string"), json ! ({"a": "Object"}))]
    #[case(json ! ("Old string"), json ! ({"a": 1}), json ! ("Old string"))]
    fn merging_as_expected(
        #[case] old: Value,
        #[case] new: Value,
        #[case] expected: Value,
    ) {
        let merged = merge(old, new);
        assert_eq!(merged, expected);
    }
}
