use std::collections::vec_deque::Drain;
use std::collections::VecDeque;
use core::time::Duration;

type Item<T> = (Duration, T);

pub type TimeVecItem<T> = Item<T>;

#[derive(Clone, Debug)]
pub struct TimeVec<T> {
    pub limit: Duration,
    buffer: VecDeque<Item<T>>,
}

impl<T> TimeVec<T> {
    pub fn new(limit: Duration, capacity: usize) -> Self {
        let buffer = VecDeque::with_capacity(capacity);
        Self { limit, buffer }
    }

    #[inline]
    pub fn checked_duration(&self) -> Option<Duration> {
        self.buffer
            .front()
            .map(|oldest| self.buffer.back().unwrap().0 - oldest.0)
    }

    #[inline]
    pub fn duration(&self) -> Duration {
        self.checked_duration().unwrap_or(Duration::ZERO)
    }

    #[inline]
    fn check_timestamp(&self, value: Duration) -> bool {
        self.buffer
            .back()
            .map(|i| value > i.0)
            .unwrap_or(true)
    }

    #[inline]
    pub fn push(&mut self, timestamp: Duration, item: T) {
        assert!(self.check_timestamp(timestamp));

        self.buffer.push_back((timestamp, item));

        let partition_timestamp = timestamp.saturating_sub(self.limit);
        self.buffer.retain(|i| i.0 < partition_timestamp);
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<Item<T>> {
        self.buffer.pop_front()
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<Item<T>> {
        self.buffer.pop_back()
    }

    #[inline]
    pub fn duration_from_back(&self, duration: &Duration) -> Option<Duration> {
        self.buffer.back().map(|item| duration.checked_sub(item.0)).flatten()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> impl ExactSizeIterator<Item = &Item<T>> + 'a {
        self.buffer.iter()
    }

    #[inline]
    pub fn iter_data<'a>(&'a self) -> impl ExactSizeIterator<Item = &T> + 'a {
        self.buffer.iter().map(|snap| &snap.1)
    }

    #[inline]
    pub fn iter_time<'a>(&'a self) -> impl ExactSizeIterator<Item = &Duration> + 'a {
        self.buffer.iter().map(|snap| &snap.0)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    #[inline]
    pub fn drain(&mut self) -> Drain<Item<T>> {
        self.buffer.drain(..)
    }
}
