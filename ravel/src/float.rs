use std::{error::Error, fmt::Display};

/// A single-value container whose value can be temporarily "floated" (moved) to
/// other locations within a closure scope.
///
/// This is an abstraction over what has been colloquially called "the option
/// dance". Ferrous Systems has a [good write-up of the general
/// pattern](https://ferrous-systems.com/blog/rustls-borrow-checker-p1/).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Float<T> {
    inner: Option<T>,
}

/// An error which occurs when attempting to use an invalid (empty) [`Float`].
///
/// This should not happen without use of [`std::panic::catch_unwind`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PoisonError;

impl Display for PoisonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "poisoned float: task unwinded while value was floated")
    }
}

impl Error for PoisonError {}

impl<T> Float<T> {
    /// Creates a new [`Float`] with the provided `value`.
    pub fn new(value: T) -> Self {
        Self { inner: Some(value) }
    }

    /// Consumes this [`Float`], returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ravel::Float;
    /// let value = "hello";
    /// let float = Float::new(value);
    /// assert_eq!(float.into_inner(), Ok(value));
    /// ```
    pub fn into_inner(self) -> Result<T, PoisonError> {
        self.inner.ok_or(PoisonError)
    }

    /// Gets a reference to the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ravel::Float;
    /// let mut float = Float::new(true);
    /// assert_eq!(float.as_ref(), Ok(&true));
    /// ```
    pub fn as_ref(&self) -> Result<&T, PoisonError> {
        self.inner.as_ref().ok_or(PoisonError)
    }

    /// Gets a mutable reference to the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ravel::Float;
    /// let mut float = Float::new("hello");
    /// *float.as_mut().unwrap() = "bye";
    /// assert_eq!(float.into_inner(), Ok("bye"));
    /// ```
    pub fn as_mut(&mut self) -> Result<&mut T, PoisonError> {
        self.inner.as_mut().ok_or(PoisonError)
    }

    /// Invokes a callback which is given access to the data by value, and can
    /// freely move it to other locations before ultimately returning a
    /// (possibly new) value to be stored, along with an extra result value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ravel::Float;
    /// fn show(pair: &(String, String)) {
    ///     println!("{} {}", pair.0, pair.1);
    /// }
    ///
    /// let mut float = Float::new("hello".to_string());
    ///
    /// let result = float.float(|value| {
    ///     let pair = (value, "world".to_string());
    ///     show(&pair);
    ///     ("bye".to_string(), true)
    /// });
    /// assert_eq!(result, Ok(true));
    /// ```
    pub fn float<F, R>(&mut self, f: F) -> Result<R, PoisonError>
    where
        F: FnOnce(T) -> (T, R),
    {
        let (value, result) = f(self.inner.take().ok_or(PoisonError)?);
        self.inner = Some(value);
        Ok(result)
    }

    /// Invokes a callback which is given access to the data by value, and can
    /// freely move it to other locations before ultimately returning a
    /// (possibly new) value to be stored.
    ///
    /// This is like [`Self::float`], but without the extra return value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ravel::Float;
    /// fn show(pair: &(String, String)) {
    ///     println!("{} {}", pair.0, pair.1);
    /// }
    ///
    /// let mut float = Float::new("hello".to_string());
    ///
    /// let result = float.float_(|value| {
    ///     let pair = (value, "world".to_string());
    ///     show(&pair);
    ///     "bye".to_string()
    /// });
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn float_<F>(&mut self, f: F) -> Result<(), PoisonError>
    where
        F: FnOnce(T) -> T,
    {
        self.float(|v| (f(v), ()))
    }
}
