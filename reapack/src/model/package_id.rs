/// Owned package ID.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PackageId {
    pub remote: String,
    pub category: String,
    pub package: String,
}

/// Borrowed package ID.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LightPackageId<'a> {
    pub remote: &'a str,
    pub category: &'a str,
    pub package: &'a str,
}

impl PackageId {
    pub fn to_borrowed(&self) -> LightPackageId {
        LightPackageId {
            remote: &self.remote,
            category: &self.category,
            package: &self.category,
        }
    }
}

impl<'a> LightPackageId<'a> {
    pub fn to_owned(&self) -> PackageId {
        PackageId {
            remote: self.remote.to_string(),
            category: self.category.to_string(),
            package: self.package.to_string(),
        }
    }
}
