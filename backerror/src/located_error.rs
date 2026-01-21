use std::ops::Deref;
use std::panic::Location;
#[cfg(any(feature = "backtrace", feature = "force_backtrace"))]
use std::sync::Arc;

/// New error type encapsulating the original error and location data.
/// ```ignore
/// fn open_fail() -> Result<(), LocatedError<std::io::Error>> {
///     std::fs::File::open("blurb.txt")?;
///     Ok(())
/// }
/// let _r = open_fail();
/// ```
pub struct LocatedError<E: std::error::Error> {
    inner: E,
    location: &'static Location<'static>,

    #[cfg(any(feature = "backtrace", feature = "force_backtrace"))]
    backtrace: Arc<std::backtrace::Backtrace>,
}

/// Error
impl<E: std::error::Error> std::error::Error for LocatedError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

/// Display
impl<E: std::error::Error> std::fmt::Display for LocatedError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PAT: &str = "; Caused by ";
        let inner_msg = format!("{}", self.inner);
        if let Some(pos) = inner_msg.find(PAT) {
            write!(
                f,
                "{}; Caused by {} ({}){}",
                &inner_msg[..pos],
                std::any::type_name::<E>(),
                self.location,
                &inner_msg[pos..]
            )
        } else {
            write!(
                f,
                "{}; Caused by {}({});",
                self.inner,
                std::any::type_name::<E>(),
                self.location,
            )
        }
    }
}

/// Debug
impl<E: std::error::Error> std::fmt::Debug for LocatedError<E> {
    #[cfg(not(any(feature = "backtrace", feature = "force_backtrace")))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let name = std::any::type_name::<E>();
        // let pos = name.rfind(":").unwrap_or(0);
        // let name = &name[pos + 1..];
        write!(
            f,
            "{:?}\n\tat ({}) by {}",
            self.inner,
            self.location,
            std::any::type_name::<E>(), // name
        )
    }

    #[cfg(any(feature = "backtrace", feature = "force_backtrace"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(stacktrace) = super::stacktrace::StackTrace::parse(&self.backtrace) {
            self.fmt_stacktrace(stacktrace, f)
        } else {
            write!(
                f,
                "{:?}\n\tat ({}) by {}",
                self.inner,
                self.location,
                std::any::type_name::<E>() // name
            )
        }
    }
}

#[cfg(any(feature = "backtrace", feature = "force_backtrace"))]
impl<E: std::error::Error> LocatedError<E> {
    fn fmt_stacktrace(
        &self,
        stacktrace: super::stacktrace::StackTrace,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        const CAUSED_BY_PAT: &str = "\nCaused by: ";

        let inner_msg = format!("{:?} ({})", self.inner, self.location);

        let index = if let Some(cause_index) = inner_msg.find(CAUSED_BY_PAT) {
            // write the head
            let msg_head = &inner_msg[0..cause_index + 1];
            write!(f, "{}", msg_head)?;
            cause_index
        } else {
            // write total message
            writeln!(f, "{}", inner_msg)?;
            0
        };

        // inject the stacktrace
        writeln!(
            f,
            "Caused by: {}: {}",
            std::any::type_name::<E>(),
            self.pure_desc()
        )?;
        for frame in stacktrace.frames {
            let line = if frame.file.is_empty() {
                format!("\tat {}", frame.func)
            } else {
                format!("\tat {} ({}:{})", frame.func, frame.file, frame.line)
            };
            if inner_msg.find(&line).is_some() {
                continue;
            }
            writeln!(f, "{}", line)?;
        }

        // write the rest
        if index > 0 {
            let msg_remain = &inner_msg[index + 1..];
            write!(f, "{}", msg_remain)?;
        }
        write!(f, "")
    }

    fn pure_desc(&self) -> String {
        const PAT: &str = "; Caused by ";

        let desc = self.inner.to_string();
        let desc = if let Some(pos) = desc.find(PAT) {
            desc[..pos].to_string()
        } else {
            desc
        };
        desc
    }
}

/// From
impl<E: std::error::Error> From<E> for LocatedError<E> {
    #[track_caller]
    fn from(err: E) -> Self {
        LocatedError {
            inner: err,
            location: std::panic::Location::caller(),

            #[cfg(all(feature = "backtrace", not(feature = "force_backtrace")))]
            backtrace: Arc::new(std::backtrace::Backtrace::capture()),

            #[cfg(feature = "force_backtrace")]
            backtrace: Arc::new(std::backtrace::Backtrace::force_capture()), // or Backtrace::disabled()
        }
    }
}

/// AsRef
impl<T: std::error::Error> AsRef<T> for LocatedError<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

/// Deref
impl<T: std::error::Error> Deref for LocatedError<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

/// Borrow
impl<T: std::error::Error> std::borrow::Borrow<T> for LocatedError<T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

// Send
unsafe impl<T: std::error::Error + Send> Send for LocatedError<T> {}

/// Sync
unsafe impl<T: std::error::Error + Sync> Sync for LocatedError<T> {}

// Clone
impl<T: std::error::Error + Clone> Clone for LocatedError<T> {
    fn clone(&self) -> Self {
        LocatedError {
            inner: self.inner.clone(),
            location: self.location,

            #[cfg(any(feature = "backtrace", feature = "force_backtrace"))]
            backtrace: self.backtrace.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use thiserror::Error;

    use super::*;
    #[test]
    fn test_located_error() {
        #[derive(Debug, Error)]
        pub enum MyError {
            #[error("{0}")]
            LocatedIoError(#[from] LocatedError<std::io::Error>),
        }

        impl From<std::io::Error> for MyError {
            #[track_caller]
            fn from(err: std::io::Error) -> Self {
                MyError::from(LocatedError::from(err))
            }
        }

        fn located_error1() -> Result<(), MyError> {
            std::fs::File::open("blurb.txt")
                .map_err(|e| LocatedError::<std::io::Error>::from(e))?;
            Ok(())
        }

        fn located_error2() -> Result<(), MyError> {
            std::fs::File::open("blurb.txt")?;
            Ok(())
        }

        if let Err(e) = located_error1() {
            println!("error {}", e);
            println!("error {:?}", e);
        }

        if let Err(e) = located_error2() {
            println!("error {}", e);
            println!("error {:?}", e);
        }
    }
}
