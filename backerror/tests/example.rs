#[cfg(test)]
mod tests {
    use backerror::backerror;
    use thiserror::Error;

    #[test]
    fn test_backerror() {
        #[backerror]
        #[derive(Debug, Error)]
        #[error(transparent)]
        pub struct MyError1(#[from] std::io::Error);

        #[backerror]
        #[derive(Debug, Error)]
        pub enum MyError2 {
            #[error(transparent)]
            MyError1(#[from] MyError1),
        }

        fn throw_error1() -> Result<(), MyError1> {
            std::fs::File::open("blurb.txt")?;
            Ok(())
        }

        fn throw_error2() -> Result<(), MyError2> {
            Ok(throw_error1()?)
        }

        if let Err(e) = throw_error2() {
            println!("======= Debug Error ======");
            println!("{:?}", e);
            println!("======= Display Error =======");
            println!("{}", e);
        }
    }
}
