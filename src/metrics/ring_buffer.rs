use serde::Serialize;
use std::collections::VecDeque;

#[derive(Clone, Serialize)]
pub struct RingBuffer<T: Clone> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buffer = RingBuffer::new(3);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.len(), 3);
        let values: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut buffer = RingBuffer::new(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4);

        assert_eq!(buffer.len(), 3);
        let values: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(values, vec![2, 3, 4]);

        buffer.push(5);
        buffer.push(6);

        assert_eq!(buffer.len(), 3);
        let values: Vec<_> = buffer.iter().copied().collect();
        assert_eq!(values, vec![4, 5, 6]);
    }

    #[test]
    fn test_ring_buffer_with_strings() {
        let mut buffer = RingBuffer::new(2);

        buffer.push("first".to_string());
        buffer.push("second".to_string());
        buffer.push("third".to_string());

        let values: Vec<_> = buffer.iter().map(|s| s.as_str()).collect();
        assert_eq!(values, vec!["second", "third"]);
    }
}
