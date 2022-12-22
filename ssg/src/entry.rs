/// Whether an entry has been newly added, if it existed but was changed, or if existed and remained unchanged.
pub enum BuildStatus {
    New,
    Changed,
    Unchanged,
}

// Describes common behavior for an entry.
//
// An entry is the main unit the static site generator works with. It can be a markdown file, stylesheet, or some other static asset.
pub trait Entry {
    type Built;

    fn build_status() -> BuildStatus;
    fn hash() -> String;
    fn build() -> Self::Built;
}
