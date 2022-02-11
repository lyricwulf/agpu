use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    mem::size_of,
};

mod marker;

mod section;

mod queryset;
pub use queryset::QuerySet;

pub const MAX_QUERIES: u32 = wgpu::QUERY_SET_MAX_QUERIES;
pub const MAX_BUFFER_SIZE: u64 = MAX_QUERIES as u64 * size_of::<u64>() as u64;

pub const PIPELINE_STATISTICS_LABELS: [&str; 5] = [
    "Vertex shader invocations",
    "Clipper invokations",
    "Clipper primitives out",
    "Fragment shader invocations",
    "Compute shader invocations",
];

#[derive(Debug)]
pub struct Profiler {
    /// 64-bit number indicating the GPU-timestamp where all previous commands have finished executing
    pub(crate) timestamp: Option<QuerySet>,
    pub(crate) stats: Option<QuerySet>,
    /// The amount of nanoseconds each tick of a timestamp query represents
    pub timestamp_period: f32,
    markers: RefCell<Vec<String>>,
    resolved: Cell<bool>,
}

impl Profiler {
    #[allow(clippy::cast_possible_truncation)]
    pub fn query_count(&self) -> u32 {
        self.markers.borrow().len() as u32
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn query_index(&self) -> u32 {
        self.query_count() - 1
    }

    #[must_use]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // Timestamp period is multiplied by the time span to get the duration in nanoseconds
        let timestamp_period = queue.get_timestamp_period();

        // QuerySet availability is based on device's feature support.
        // If you want to opt in or out of a query set then do so in the feature set.
        let timestamp = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| QuerySet::new_timestamp(device, MAX_QUERIES));

        let stats = device
            .features()
            .contains(wgpu::Features::PIPELINE_STATISTICS_QUERY)
            .then(|| QuerySet::new_stats(device, MAX_QUERIES));

        Self {
            timestamp,
            stats,
            timestamp_period,
            markers: RefCell::new(Vec::new()),
            resolved: Cell::new(false),
        }
    }

    pub(crate) fn begin_section(&self, label: &str) {
        self.markers.borrow_mut().push(label.to_string());
    }

    pub fn timestamp(&self, _label: &str, encoder: &mut wgpu::CommandEncoder) {
        if let Some(ts_qs) = &self.timestamp {
            encoder.write_timestamp(ts_qs, self.query_index());
        }
    }

    pub fn begin_stats(&self, render_pass: &mut wgpu::RenderPass) {
        if let Some(stats_qs) = &self.stats {
            render_pass.begin_pipeline_statistics_query(stats_qs, self.query_index());
        }
    }

    pub fn end_stats(&self, render_pass: &mut wgpu::RenderPass) {
        if self.stats.is_some() {
            render_pass.end_pipeline_statistics_query();
        }
    }

    /// Must be called before get()
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        self.stats
            .as_ref()
            .unwrap()
            .resolve(self.query_count(), encoder);
        // if !self.resolved.replace(true) {
        //     // If replace() returns false then the query set still needs to be resolved
        //     self.foreach_query_set(|query_set| query_set.resolve(self.query_count(), encoder));
        // }
    }

    pub fn timestamp_report(&self, device: &wgpu::Device) -> Vec<(String, f32)> {
        let mut ret = vec![];
        if let Some(timestamp) = &self.timestamp {
            if let Ok(val) = timestamp.get(device, self.query_count()) {
                for (i, marker) in self.markers.borrow()[1..].iter().enumerate() {
                    let start = val[i];
                    let end = val[i + 1];
                    let duration = self.ts_to_millis(end - start);
                    // println!("{} took {} ms", marker, duration / 1_000_000.0);
                    ret.push((marker.clone(), duration / 1_000_000.0));
                }
            }
        };

        // if let Some(stats) = &self.stats {
        //     if let Ok(val) = stats.get(device, self.query_count()) {
        //         println!("Stats: {:#?}", val);
        //     }
        // };

        ret
    }

    #[deprecated]
    #[allow(dead_code)]
    fn foreach_query_set<F, T>(&self, mut f: F) -> Vec<T>
    where
        F: FnMut(&QuerySet) -> T,
    {
        let sets = &[self.timestamp.as_ref(), self.stats.as_ref()];
        let iter = sets.iter().flatten();
        iter.map(|q| f(*q)).collect::<Vec<_>>()
    }

    pub fn clear(&self) {
        self.markers.borrow_mut().clear();
        self.resolved.set(false);
    }

    #[allow(clippy::cast_precision_loss)]
    fn ts_to_millis(&self, ts: u64) -> f32 {
        ts as f32 * self.timestamp_period / 1_000_000.0
    }
}
