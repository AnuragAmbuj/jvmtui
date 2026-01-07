use crate::jvm::types::{ClassInfo, GcStats, HeapInfo, ThreadInfo};
use crate::metrics::ring_buffer::RingBuffer;

#[derive(Clone)]
pub struct MetricsStore {
    pub heap_history: RingBuffer<HeapInfo>,
    pub gc_history: RingBuffer<GcStats>,
    pub thread_snapshot: Vec<ThreadInfo>,
    pub class_histogram: Vec<ClassInfo>,
}

impl MetricsStore {
    pub fn new(history_size: usize) -> Self {
        Self {
            heap_history: RingBuffer::new(history_size),
            gc_history: RingBuffer::new(history_size),
            thread_snapshot: Vec::new(),
            class_histogram: Vec::new(),
        }
    }

    pub fn record_heap(&mut self, info: HeapInfo) {
        self.heap_history.push(info);
    }

    pub fn record_gc(&mut self, stats: GcStats) {
        self.gc_history.push(stats);
    }

    pub fn record_threads(&mut self, threads: Vec<ThreadInfo>) {
        self.thread_snapshot = threads;
    }

    pub fn record_class_histogram(&mut self, classes: Vec<ClassInfo>) {
        self.class_histogram = classes;
    }
}
