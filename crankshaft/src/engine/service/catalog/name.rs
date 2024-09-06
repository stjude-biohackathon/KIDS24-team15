//! Names for registered services with the catalog.

/// The character to split the namespace and the named service.
const SEPARATOR: &str = "/";

/// An error related to parsing a [`Name`].
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ParseError(String);

impl ParseError {
    /// Creates a new [`ParseError`] from the provided value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// A name for a registered service within the catalog.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Name {
    /// A key to a named logging service.
    Logging(String),

    /// A key to a named task runner service.
    Runner(String),
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // NOTE: if you add an option here, be sure to also add it to
            // [`FromStr`] below (and add a test)!
            Name::Logging(name) => write!(f, "{}{}{}", "logging", SEPARATOR, name),
            Name::Runner(name) => write!(f, "{}{}{}", "runner", SEPARATOR, name),
        }
    }
}

impl std::str::FromStr for Name {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(SEPARATOR).collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(ParseError(format!(
                "invalid number of components: {}",
                parts.len()
            )));
        }

        let mut parts = parts.into_iter();

        // SAFETY: we just ensured above we have exactly two parts, so these
        // will always unwrap.
        let namespace = parts.next().unwrap();
        let local_name = parts.next().unwrap();

        match namespace {
            // NOTE: if you add an option here, be sure to also add it to
            // [`std::fmt::Display`] above (and add a test)!
            "logging" => Ok(Name::Logging(String::from(local_name))),
            "runner" => Ok(Name::Runner(String::from(local_name))),
            _ => Err(ParseError(format!("unknown namespace: {}", namespace))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(
            "logging/name".parse::<Name>().unwrap(),
            Name::Logging(String::from("name"))
        );

        assert_eq!(
            "runner/name".parse::<Name>().unwrap(),
            Name::Runner(String::from("name"))
        );

        let err = "runner".parse::<Name>().unwrap_err();
        assert_eq!(err, ParseError::new("invalid number of components: 1"));

        let err = "runner/foo/bar".parse::<Name>().unwrap_err();
        assert_eq!(err, ParseError::new("invalid number of components: 3"));

        let err = "foo/name".parse::<Name>().unwrap_err();
        assert_eq!(err, ParseError::new("unknown namespace: foo"));
    }
}
