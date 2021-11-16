#[derive(Debug)]
pub struct TimestampMarker<'a> {
    pub label: &'a str,
    pub duration: f32,
}
