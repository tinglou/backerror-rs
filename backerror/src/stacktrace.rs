use std::backtrace::{Backtrace, BacktraceStatus};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StackTrace {
    pub frames: Vec<StackTraceFrame>,
}

#[derive(Debug, Deserialize)]
pub struct StackTraceFrame {
    #[serde(rename = "fn")]
    pub func: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: u32,
}

impl StackTrace {
    /// parse [`Backtrace`]'s debug output
    pub fn parse(backtrace: &Backtrace) -> Option<Self> {
        if backtrace.status() != BacktraceStatus::Captured {
            return None;
        }

        let msg = format!("{:?}", backtrace);
        let offset = "Backtrace ".len();
        let msg = format!(r#"{{"frames": {}}}"#, &msg[offset..]);

        match json5::from_str::<StackTrace>(&msg) {
            Ok(mut stacktrace) => {
                stacktrace.nomalize();
                Some(stacktrace)
            }
            Err(_) => None,
        }
    }

    /// parse [`Backtrace`]'s debug output
    /// ```txt
    /// Backtrace [{ fn: "std::backtrace_rs::backtrace::win64::trace", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\..\..\backtrace\src\backtrace\win64.rs", line: 85 }, { fn: "std::backtrace_rs::backtrace::trace_unsynchronized", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\..\..\backtrace\src\backtrace\mod.rs", line: 66 }, { fn: "std::backtrace::Backtrace::create", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\backtrace.rs", line: 331 }, { fn: "std::backtrace::Backtrace::force_capture", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\backtrace.rs", line: 312 }, { fn: "backerror::stacktrace::tests::parse_backtrace", file: ".\src\stacktrace.rs", line: 118 }, { fn: "backerror::stacktrace::tests::parse_backtrace::closure$0", file: ".\src\stacktrace.rs", line: 117 }, { fn: "core::ops::function::FnOnce::call_once<backerror::stacktrace::tests::parse_backtrace::closure_env$0,tuple$<> >", file: "C:\Users\admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\ops\function.rs", line: 250 }, { fn: "core::ops::function::FnOnce::call_once", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\core\src\ops\function.rs", line: 250 }, { fn: "test::__rust_begin_short_backtrace<enum2$<core::result::Result<tuple$<>,alloc::string::String> >,enum2$<core::result::Result<tuple$<>,alloc::string::String> > (*)()>", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs", line: 663 }, { fn: "test::run_test_in_process", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs", line: 686 }, { fn: "test::run_test::closure$0", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs", line: 607 }, { fn: "test::run_test::closure$1", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\test\src\lib.rs", line: 637 }, { fn: "std::sys::backtrace::__rust_begin_short_backtrace<test::run_test::closure_env$1,tuple$<> >", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\sys\backtrace.rs", line: 158 }, { fn: "core::ops::function::FnOnce::call_once<std::thread::impl$0::spawn_unchecked_::closure_env$1<test::run_test::closure_env$1,tuple$<> >,tuple$<> >", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\core\src\ops\function.rs", line: 250 }, { fn: "alloc::boxed::impl$29::call_once", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\alloc\src\boxed.rs", line: 1985 }, { fn: "alloc::boxed::impl$29::call_once", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\alloc\src\boxed.rs", line: 1985 }, { fn: "std::sys::thread::windows::impl$0::new::thread_start", file: "/rustc/f8297e351a40c1439a467bbbb6879088047f50b3/library\std\src\sys\thread\windows.rs", line: 60 }, { fn: "BaseThreadInitThunk" }, { fn: "RtlUserThreadStart" }]
    /// ```
    fn parse_debug_str(debug: &str) -> Option<Self> {
        None
    }

    /// normalize stacktrace
    /// 1. remove starting frames owned by [`Backtrace`]
    /// 2. remove the leading prefix from file path
    fn nomalize(&mut self) {
        //  1. remove starting frames owned by [`Backtrace`]
        loop {
            if let Some(first) = self.frames.first() {
                if first.func.starts_with("std::backtrace")
                    || first.func.starts_with("<backerror::")
                {
                    self.frames.remove(0);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // 2. remove the leading prefix from file path
        // 2.1 reverse find possible source dir, "src", "tests", "bench", "examples"
        // 2.2 get the crate name, remove all path parts before it
        for frame in self.frames.iter_mut() {
            #[cfg(windows)]
            let src_pat = vec!["\\src\\", "\\tests\\", "\\bench\\", "\\examples\\"];
            #[cfg(not(windows))]
            let src_pat = vec!["/src/", "/tests/", "/bench/", "/examples/"];
            let mut max_index = 0;
            for (_i, pat) in src_pat.iter().enumerate() {
                if let Some(index) = Self::find_crate_name_offset(&frame.file, pat) {
                    if index > max_index {
                        max_index = index;
                    }
                }
            }
            if max_index > 0 {
                frame.file = frame.file[max_index..].to_string();
            }
        }
    }

    /// "/home/user/project/src/main.rs" => "project/src/main.rs"
    fn find_crate_name_offset(path: &str, src_pat: &str) -> Option<usize> {
        // firstly find position of "/src/"
        if let Some(src_index) = path.rfind(src_pat) {
            // find the nestest position of "/"
            if let Some(slash_index) = path[..src_index].rfind('/') {
                Some(slash_index + 1)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StackTrace;

    #[test]
    fn find_crate_name_offset() {
        let path = "/home/user/project/src/main.rs";
        let src_pat = "/src/";
        let index = StackTrace::find_crate_name_offset(path, src_pat).unwrap();

        println!("index: {}, substr: {}", index, &path[index..]);

        let path = "project/src/main.rs";
        let src_pat = "/src/";
        let index = StackTrace::find_crate_name_offset(path, src_pat).unwrap();

        println!("index: {}, substr: {}", index, &path[index..]);
    }

    #[test]
    fn parse_backtrace() {
        let backtrace = std::backtrace::Backtrace::force_capture();

        println!("{:?}", backtrace);

        println!("{}", backtrace);

    }
}
