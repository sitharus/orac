pub fn or_empty<T>(s: &Option<T>) -> ::askama::Result<String>
where
    T: AsRef<str>,
{
    match s {
        Some(s) => Ok(s.as_ref().into()),
        _ => Ok("".into()),
    }
}
