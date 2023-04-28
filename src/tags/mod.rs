use std::collections::BTreeMap;

pub fn parse_simple_tags(message: String, values: BTreeMap<String, String>) -> String {
    let mut chars = message.chars();
    let mut result = String::with_capacity(message.len());

    while let Some(symbol) = chars.next() {
        if symbol == '{' {
            let mut name = String::new();
            for sym in chars.by_ref() {
                if sym == '}' {
                    break;
                } else {
                    name.push(sym);
                }
            }

            let value = values.get(&name);
            if let Some(val) = value {
                result.push_str(val);
            }
        } else {
            result.push(symbol);
        }
    }

    result
}
