pub fn ascii_lowercase(string: &str) -> ~str {
    let mut result = ~"";
    for c in string.iter() {
        result.push_char((match c {
                    '\x41'..'\x5a' => c | '\x20',
                    _ => c
                }))
    };
    result
}

enum XMLNameType {
    QName,
    Name,
    Invalid
}

pub fn xml_name_type(string: &str) -> XMLNameType {
    //TODO
    QName
}
