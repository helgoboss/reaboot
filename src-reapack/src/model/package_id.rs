#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LightPackageId<'a> {
    pub remote: &'a str,
    pub category: &'a str,
    pub package: &'a str,
}
