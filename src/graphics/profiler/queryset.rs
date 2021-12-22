use std::{mem::size_of, ops::Deref};

use futures::executor::block_on;

use crate::QUERYSET_BUFFER_USAGE;
use crate::{GpuError, ScopedBufferView};

pub struct QuerySet {
    pub(crate) inner: wgpu::QuerySet,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) ty: wgpu::QueryType,
}
/// Allows deref of a `QuerySet` to the inner `wgpu::QuerySet`.
impl Deref for QuerySet {
    type Target = wgpu::QuerySet;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl QuerySet {
    /// Creates a new `QuerySet` that times various events.
    pub(crate) fn new_timestamp(device: &wgpu::Device, count: u32) -> Self {
        let ty = wgpu::QueryType::Timestamp;

        let label = Some("Timestamp QuerySet");
        Self::new_impl(device, ty, label, count)
    }

    /// Creates a new `QuerySet` that counts various pipeline events.
    /// See `QueryType::PipelineStatistics`
    ///
    /// ! For now, this always uses all pipeline stat types.
    pub(crate) fn new_stats(device: &wgpu::Device, count: u32) -> Self {
        // Use all pipeline stat types
        let all = wgpu::PipelineStatisticsTypes::all();
        let ty = wgpu::QueryType::PipelineStatistics(all);

        let label = Some("PipelineStatistics QuerySet");
        Self::new_impl(device, ty, label, count)
    }

    /// Makes the wgpu calls to create the query set.
    fn new_impl(
        device: &wgpu::Device,
        ty: wgpu::QueryType,
        label: Option<&str>,
        count: u32,
    ) -> Self {
        // Size of a single query result in u64s.
        let query_size = query_ty_size(ty);
        let buffer_size = query_size * count * size_of::<u64>() as u32;

        // Create the query set
        let inner = device.create_query_set(&wgpu::QuerySetDescriptor { label, ty, count });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: buffer_size as u64,
            usage: QUERYSET_BUFFER_USAGE,
            mapped_at_creation: false,
        });
        QuerySet { inner, buffer, ty }
    }

    pub fn query_size(&self) -> u32 {
        query_ty_size(self.ty)
    }

    pub fn resolve(&self, count: u32, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(self, 0..count * self.query_size(), &self.buffer, 0)
    }

    /// Must first call resolve()
    pub fn get(&self, device: &wgpu::Device, count: u32) -> Result<Vec<u64>, GpuError> {
        // Check to see if there aren't any markers
        if count == 0 {
            // This is not an error
            // We return an empty array to avoid mapping the buffer which we already know is empty
            return Ok(Vec::new());
        }

        // Map the buffer for reading
        // ? Maybe we can save a local copy of the last data we read so we can avoid mapping the buffer multiple times
        let slice = self
            .buffer
            .slice(..size_of::<u64>() as u64 * (count * self.query_size()) as u64);
        let mapping = slice.map_async(wgpu::MapMode::Read);
        device.poll(wgpu::Maintain::Wait);
        block_on(mapping).map_err(|_| GpuError::BufferAsyncError)?;
        let view = slice.get_mapped_range();

        let view = ScopedBufferView::new(&self.buffer, view);
        let timestamps: &[u64] = bytemuck::cast_slice(&view);

        // dbg!(&timestamps);

        Ok(timestamps.to_vec())
    }
}

/// The size (in u64s) of a single query result given the query type.
fn query_ty_size(ty: wgpu::QueryType) -> u32 {
    // Size of a single query.
    match ty {
        wgpu::QueryType::PipelineStatistics(ty) => num_bits_set(ty.bits()),
        _ => 1,
    }
}

/// Counts the number of bits set for the input n. Used for bitflags.
fn num_bits_set<N>(n: N) -> u32
where
    N: num_traits::PrimInt,
{
    n.count_ones()
}
