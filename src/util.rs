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

// Attempt to decode bytes as ASCII, replacing invalid characters with a _.
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        if *b >= 32 && *b <= 126 {
            s.push(*b as char);
        } else {
            s.push('_');
        }
    }
    s
}

pub fn empty_vec_as_none<T>(vec: Vec<T>) -> Option<Vec<T>> {
    match vec.is_empty() {
        true => None,
        false => Some(vec),
    }
}

pub fn empty_str_as_none(s: String) -> Option<String> {
    match s.is_empty() {
        true => None,
        false => Some(s),
    }
}
