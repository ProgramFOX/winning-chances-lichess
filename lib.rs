pub fn calculate_from_files<'a, I>(files: I) -> String
where
    I: IntoIterator<Item = &'a str>
{
    String::from("ok")
}