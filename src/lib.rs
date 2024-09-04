//! Crankshaft.

pub mod engine;

/// Generates the `as_`, `into_`, and `unwrap_` methods commonly found on enum
/// wrappers.o
///
/// # Args
///
/// * `suffix` is the suffix for the method names generated (e.g. `logger` if
///   you want `as_logger()`, `into_logger()`, and `unwrap_logger` to be
///   generated).
/// * `inner` is the name of the inner type wrapped by the enum variant (e.g.,
///   `Logger`) if that is the construct struct wrapped.
/// * `variant` is the name of the variant in the enum that wraps the concrete
///   struct (e.g., `Logger` if the variant is `Self::Logger`).
///
/// A simple example:
///
/// ```rust
/// /// Services that can be registered within the engine.
/// #[derive(Debug)]
/// pub enum Service {
///     /// A logging service.
///     Logger(Logger),
///
///     /// A task runner service.
///     Runner(Runner),
/// }
///
/// impl Service {
///     as_into_unwrap!(logger, Logger, Logger);
///     as_into_unwrap!(runner, Runner, Runner);
/// }
/// ```
#[macro_export]
macro_rules! as_into_unwrap {
    ($suffix:ident, $inner:ty, $variant:ident) => {
        paste::paste! {
                    #[doc = "Attempts to get a reference to the inner [`" $inner "`].
\
                    * If `self` is a [`Self::" $variant "`], then a reference to the inner [`" $inner "`] wrapped in [`Some`] is returned.
\
                    * Else, [`None`] is returned."
                    ]
                    pub fn [<as_ $suffix>](&self) -> Option<&$inner> {
                        match self {
                            Self::$variant($suffix) => Some($suffix),
                            _ => None,
                        }
                    }

                    #[doc = "Consumes `self` and attempts to return the inner [`" $inner "`].
\
                    * If `self` is a [`Self::" $variant "`], then the inner [`" $inner "`] wrapped in [`Some`] is returned.
\
                    * Else, [`None`] is returned."
                    ]
                    pub fn [<into_ $suffix>](self) -> Option<$inner> {
                        match self {
                            Self::$variant($suffix) => Some($suffix),
                            _ => None,
                        }
                    }

                    #[doc = "Consumes `self` and returns the inner [`" $inner "`].
\
# Panics
\
                    If `self` is not a [`Self::" $variant "`]."
                    ]
                    pub fn [<unwrap_ $suffix>](self) -> $inner {
                        self.[<into_ $suffix>]().expect(
                            "expected `${stringify!($variant)}` but got a different variant"
                        )
                    }
                }
    };
}
