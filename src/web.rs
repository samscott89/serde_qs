use std::fmt;
use std::ops::{Deref, DerefMut};

/// Extract typed information from the request's form data.
///
/// Defaults to deserializing using `querystring` encoding.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QsQuery<T>(pub T);

impl<T> QsQuery<T> {
    /// Unwrap into inner T value
    pub fn into_inner(self) -> T {
        self.0
    }
}
impl<T> Deref for QsQuery<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for QsQuery<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for QsQuery<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Extract typed information from the request's form data.
///
/// Defaults to deserializing using `application/x-www-form-urlencoded` encoding.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QsForm<T>(pub T);

impl<T> QsForm<T> {
    /// Unwrap into inner T value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for QsForm<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for QsForm<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for QsForm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for QsForm<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
