pub fn or_empty<T>(s: &Option<T>) -> ::askama::Result<String>
where
    T: AsRef<str>,
{
    match s {
        Some(s) => Ok(s.as_ref().into()),
        _ => Ok("".into()),
    }
}

pub fn string_or_empty<T>(s: &Option<T>) -> ::askama::Result<String>
where
    T: ToString,
{
    match s {
        Some(s) => Ok(s.to_string()),
        _ => Ok("".into()),
    }
}

pub fn or_else<T, U>(s: &Option<T>, o: U) -> ::askama::Result<String>
where
    T: ToString,
    U: ToString,
{
    match s {
        Some(x) if x.to_string() == "" => Ok(o.to_string()),
        Some(s) => Ok(s.to_string()),
        None => Ok(o.to_string()),
    }
}
