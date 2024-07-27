use std::fmt;

#[derive(Debug)]
pub struct UnpackErr {
    pub message: String
}

impl fmt::Display for UnpackErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn print_error() {
        let error = UnpackErr { message: String::from("Hello Error") };
        println!("{}", error);
    }
}
