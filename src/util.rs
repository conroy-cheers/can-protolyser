pub fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

pub fn hex_to_str<T: AsRef<[u8]>>(data: T) -> String {
    let strings = data.as_ref().iter().map(|b| format!("{:02X}", b));
    let it = strings.into_iter();

    let s = it.fold(String::new(), |mut a, b| {
        a.reserve(b.len() + 1);
        a.push_str(&b);
        a.push_str(" ");
        a
    });
    s.trim_end().to_string()
}
