#[cfg(test)]
mod tests {
    use std::error::Error;

    use backerror::backerror;
    use thiserror::Error;

    #[backerror]
    #[derive(Debug, Error)]
    #[error(transparent)]
    pub struct MyError1(#[from] std::io::Error);

    #[backerror]
    #[derive(Debug, Error)]
    pub enum MyError2 {
        #[error("MyError2 {0}")]
        MyError1(#[from] MyError1),
    }

    #[backerror]
    #[derive(Debug, Error)]
    pub enum MyError3 {
        #[error("MyError3 {0}")]
        MyError2(#[from] MyError2),
    }

    fn throw_error1() -> Result<(), MyError1> {
        std::fs::File::open("blurb.txt")?;
        Ok(())
    }

    fn throw_error2() -> Result<(), MyError2> {
        Ok(throw_error1()?)
    }
    fn throw_error3() -> Result<(), MyError3> {
        Ok(throw_error2()?)
    }

    #[test]
    fn test_display() {
        if let Err(err) = throw_error3() {
            println!("{}", err);
            let mut source = err.source();
            while let Some(e) = source {
                println!("Source: {}", e);
                source = e.source();
            }
        }
    }

    #[test]
    fn test_debug() {
        if let Err(e) = throw_error3() {
            println!("{:?}", e);
        }
    }

    #[test]
    fn test_unwrap() {
        throw_error2().unwrap();
    }
}
