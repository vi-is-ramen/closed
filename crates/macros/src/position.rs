use proc_macro::Span;

pub fn position() -> String {
    // let mut hasher = std::hash::DefaultHasher::new();
    let span = Span::call_site();
    let mut file = "".to_owned();
    let mut buf = [0u8; 4];
    for ch in &mut span.file().chars() {
        file += if !ch.is_alphanumeric() {
            "_"
        } else {
            ch.encode_utf8(&mut buf)
        };
    }
    format!("{}_{}_{}", file, span.line(), span.column())
}
