use tokenizer::Tokenizer;

mod tokenizer;

pub struct Parser<'a> {
    /// todo: let's make it private
    pub tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let tokenizer = Tokenizer::new(source);
        Self { tokenizer }
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
