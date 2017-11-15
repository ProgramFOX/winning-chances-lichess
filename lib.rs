use std::collections::HashMap;

pub type Dataset = HashMap<u32, Datapoint>;

pub struct Datapoint {
    pub value: u32,
    pub total: u32,
}

impl Datapoint {
    pub fn percentage_value(&self) -> f64 {
        (self.value as f64) / (self.total as f64)
    }
}

pub fn calculate_from_files<'a, I>(files: I) -> String
where
    I: IntoIterator<Item = &'a str>
{
    String::from("ok")
}