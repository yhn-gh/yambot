use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Command {
    name: String,
    args: Option<Arc<[String]>>,
}

impl Command {
    pub fn new(name: String, args: Option<Arc<[String]>>) -> Self {
        Self { name, args }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub trait Parser: Sized {
    type Item;

    fn parse(&self, c: &Command) -> Option<Self::Item>;
}
