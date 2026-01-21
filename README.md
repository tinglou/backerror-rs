# backerror

`backerror` is a Rust library that enhances error handling by automatically capturing location information and stack traces for errors. It provides a seamless integration with [thiserror](https://github.com/dtolnay/thiserror) to improve debugging capabilities by tracking where errors originate in your code.

## Features

- **Automatic Location Tracking**: Automatically captures the source location (file, line, column) where an error originates
- **Stack Trace Capture**: Optionally captures full stack traces for better debugging
- **Seamless Integration**: Works with existing `thiserror`-based error types
- **Zero-cost Abstraction**: Only incurs runtime cost when enabled (can be disabled in release builds)
- **Transparent Wrapping**: Preserves the original error type while adding location metadata
- **Conditional Compilation**: Can be disabled in release builds to minimize overhead

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
backerror = { git = "https://github.com/your-repo/backerror-rs" }
thiserror = "2.0"
```

## Usage

### Example Codes

```rust
    use backerror::backerror;
    use thiserror::Error;

    #[backerror]
    #[derive(Debug, Error)]
    #[error(transparent)]
    pub struct MyError1(#[from] std::io::Error);

    #[backerror]
    #[derive(Debug, Error)]
    pub enum MyError2 {
        #[error("By MyError2: {0}")]
        MyError1(#[from] MyError1),
    }

    #[backerror]
    #[derive(Debug, Error)]
    pub enum MyError3 {
        #[error("By MyError3: {0}")]
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
        }
    }

    #[test]
    fn test_debug() {
        if let Err(e) = throw_error3() {
            println!("{:?}", e);
        }
    }
```

### Example Output

#### Display Output(`to_string`)

```text
By MyError3: By MyError2: The system cannot find the file specified. (os error 2); Caused by example::tests::MyError2 (backerror\tests\example.rs:36:12); Caused by example::tests::MyError1 (backerror\tests\example.rs:33:12); Caused by std::io::error::Error(backerror\tests\example.rs:28:9);
```

#### Debug Output

```text
MyError2(MyError1(MyError1(Os { code: 2, kind: NotFound, message: "The system cannot find the file specified." } (backerror\tests\example.rs:27:9)
Caused by: example::tests::MyError2: By MyError2: The system cannot find the file specified. (os error 2)
        at example::tests::impl$10::from (.\tests\example.rs:19)
        at core::result::impl$28::from_residual<tuple$<>,enum2$<example::tests::MyError2>,enum2$<example::tests::MyError3> > (C:\Users\admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs:2177)
Caused by: example::tests::MyError1: The system cannot find the file specified. (os error 2)
        at example::tests::impl$5::from (.\tests\example.rs:12)
        at core::result::impl$28::from_residual<tuple$<>,example::tests::MyError1,enum2$<example::tests::MyError2> > (C:\Users\admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs:2177)
Caused by: std::io::error::Error: The system cannot find the file specified. (os error 2)
        at example::tests::impl$0::from (.\tests\example.rs:7)
        at core::result::impl$28::from_residual<tuple$<>,std::io::error::Error,example::tests::MyError1> (C:\Users\admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\result.rs:2177)
        at example::tests::throw_error1 (.\tests\example.rs:27)
        at example::tests::throw_error2 (.\tests\example.rs:32)
        at example::tests::throw_error3 (.\tests\example.rs:35)
        at example::tests::test_debug (.\tests\example.rs:47)
        at example::tests::test_debug::closure$0 (.\tests\example.rs:46)
        at core::ops::function::FnOnce::call_once<example::tests::test_debug::closure_env$0,tuple$<> > (C:\Users\admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\ops\function.rs:250)
        at core::ops::function::FnOnce::call_once (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\core\src\ops\function.rs:250)
        at test::__rust_begin_short_backtrace<enum2$<core::result::Result<tuple$<>,alloc::string::String> >,enum2$<core::result::Result<tuple$<>,alloc::string::String> > (*)()> (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs:663)
        at test::run_test_in_process (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs:686)
        at test::run_test::closure$0 (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs:607)
        at test::run_test::closure$1 (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs:637)
        at std::sys::backtrace::__rust_begin_short_backtrace<test::run_test::closure_env$1,tuple$<> > (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\sys\backtrace.rs:158)
        at core::ops::function::FnOnce::call_once<std::thread::impl$0::spawn_unchecked_::closure_env$1<test::run_test::closure_env$1,tuple$<> >,tuple$<> > (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\core\src\ops\function.rs:250)
        at alloc::boxed::impl$29::call_once (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\alloc\src\boxed.rs:1985)
        at alloc::boxed::impl$29::call_once (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\alloc\src\boxed.rs:1985)
        at std::sys::thread::windows::impl$0::new::thread_start (/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\sys\thread\windows.rs:60)
        at BaseThreadInitThunk
        at RtlUserThreadStart
) (backerror\tests\example.rs:32:12)) (backerror\tests\example.rs:35:12))
```

## Rust Features

The crate provides several optional features:

- `backtrace`: Enables backtrace capture only when `RUST_BACKTRACE` environment variable is set
- `force_backtrace`: Forces backtrace capture regardless of environment variables (enabled by default)
- `release_off`: Disables the backerror transformation in release builds (enabled by default)

To customize features:

```toml
[dependencies]
backerror = { version = "...", default-features = false, features = ["force_backtrace"] }
```


## How It Works

The `backerror` crate works by:

1. Providing a `#[backerror]` attribute macro that transforms your error types
2. Converting `#[from] T` attributes to `#[from] LocatedError<T>` 
3. Using Rust's `#[track_caller]` attribute to capture the location where errors are converted
4. Optionally capturing a full backtrace when the error is created

When an error occurs, it gets wrapped in a `LocatedError<T>` struct that preserves the original error while adding location metadata.

## License

Apache License, Version 2.0 [LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0)
