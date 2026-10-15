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
