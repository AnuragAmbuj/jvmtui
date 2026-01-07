use crate::jvm::types::{GcStats, HeapInfo};
use crate::metrics::ring_buffer::RingBuffer;

#[derive(Clone)]
pub struct MetricsStore {
    pub heap_history: RingBuffer<HeapInfo>,
    pub gc_history: RingBuffer<GcStats>,
}

impl MetricsStore {
    pub fn new(history_size: usize) -> Self {
        Self {
            heap_history: RingBuffer::new(history_size),
            gc_history: RingBuffer::new(history_size),
        }
    }

    pub fn record_heap(&mut self, info: HeapInfo) {
        self.heap_history.push(info);
    }

    pub fn record_gc(&mut self, stats: GcStats) {
        self.gc_history.push(stats);
    }
}
