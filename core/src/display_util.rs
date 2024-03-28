use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Separated<T> {
    pub separator: &'static str,
    pub value: T,
}

impl<T> Separated<T> {
    pub fn new(value: T, separator: &'static str) -> Self {
        Self { separator, value }
    }
}

impl<F, I, D> Display for Separated<F>
where
    F: Fn() -> I,
    I: Iterator<Item = D>,
    D: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let sep = self.separator;
        for (i, item) in (self.value)().enumerate() {
            if i == 0 {
                item.fmt(f)?;
            } else {
                write!(f, "{sep}{}", item)?;
            }
        }
        Ok(())
    }
}
