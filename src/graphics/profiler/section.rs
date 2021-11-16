#![allow(dead_code)]
///! Dead code is allowed because this is currently unused
use crate::Profiler;

/// A convenience struct for creating a profiler section.
pub struct Section<'a> {
    pub(crate) profiler: &'a Profiler,
    pub(crate) label: String,
}
impl<'a> Section<'a> {
    pub fn new(profiler: &'a Profiler, label: &'a str) -> Section<'a> {
        let label = label.to_string();
        Section { profiler, label }
    }
}

impl Drop for Section<'_> {
    fn drop(&mut self) {
        // Unused
    }
}
