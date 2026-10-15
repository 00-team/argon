pub fn snake_to_pascal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for w in value.split('_') {
        if w.is_empty() {
            continue;
        }
        out.push_str(&w[..1].to_uppercase());
        out.push_str(&w[1..]);
    }
    out
}

pub fn pascal_to_snake(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 2);
    for (i, ch) in value.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            out.push('_');
        }
        out.push(ch);
    }

    if let Some(out) = out.strip_suffix('_') { out.to_string() } else { out }
}

pub fn default_name(ident: &str) -> String {
    let mut ident = pascal_to_snake(ident).to_uppercase();
    ident.push_str("_DEF");
    ident
}
