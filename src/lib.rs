use std::collections::vec_deque::Drain;
use std::collections::VecDeque;
use core::time::Duration;
use std::marker::PhantomData;

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

    pub fn builder() -> TimeVecBuilder<T> {
        TimeVecBuilder::<T>::default()
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
    pub fn push(&mut self, timestamp: Duration, item: T) -> Option<Drain<Item<T>>> {
        if self.check_timestamp(timestamp) {
            self.buffer.push_back((timestamp, item));

            let partition_timestamp = timestamp.saturating_sub(self.limit);
            let partition_point = self.buffer.partition_point(|i| i.0 < partition_timestamp);

            Some(self.buffer.drain(0..partition_point))
        } else {
            None
        }
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

#[derive(Copy, Clone, Debug)]
pub struct TimeVecBuilder<T> {
    pub limit: Option<Duration>,
    pub capacity: Option<usize>,
    _item: PhantomData<T>,
}

impl<T> Default for TimeVecBuilder<T> {
    fn default() -> Self {
        Self { limit: None, capacity: None, _item: PhantomData }
    }
}

impl<T> TimeVecBuilder<T> {
    pub fn with_limit(mut self, value: Duration) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn with_limit_secs(self, value: u64) -> Self {
        self.with_limit(Duration::from_secs(value))
    }

    pub fn with_limit_micros(self, value: u64) -> Self {
        self.with_limit(Duration::from_micros(value))
    }

    pub fn with_limit_millis(self, value: u64) -> Self {
        self.with_limit(Duration::from_millis(value))
    }

    pub fn with_limit_nanos(self, value: u64) -> Self {
        self.with_limit(Duration::from_nanos(value))
    }

    pub fn with_capacity(mut self, value: usize) -> Self {
        self.capacity = Some(value);
        self
    }

    pub fn build(self) -> TimeVec<T> {
        TimeVec {
            limit: self.limit.unwrap_or_default(),
            buffer: self.capacity
                .map(|value| VecDeque::<Item<T>>::with_capacity(value))
                .unwrap_or_default()
                
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn with_zero_limit() {
        let mut tv = TimeVec::<()>::builder()
            .with_limit(Duration::ZERO)
            .build();
        assert!(tv.is_empty());

        assert_eq!(tv.checked_duration(), None);
        assert_eq!(tv.duration(), Duration::ZERO);
        assert_eq!(tv.len(), 0);

        tv.push(Duration::from_secs(1), ());
        assert_eq!(tv.checked_duration(), Some(Duration::ZERO));
        assert_eq!(tv.duration(), Duration::ZERO);
        assert_eq!(tv.len(), 1);
    }

    #[test]
    fn push_two_items_with_same_timestamp() {
        let mut tv = TimeVec::<()>::builder()
            .with_limit_nanos(1)
            .build();
        
        tv.push(Duration::from_secs(1), ());
        assert_eq!(tv.len(), 1);

        tv.push(Duration::from_secs(1), ());
        assert_eq!(tv.len(), 1);
    }

    #[test]
    fn min_limit() {
        let mut tv = TimeVec::<()>::builder()
            .with_limit_nanos(1)
            .build();
        
        tv.push(Duration::ZERO, ());
        assert_eq!(tv.len(), 1);

        tv.push(Duration::from_nanos(1), ());
        assert_eq!(tv.len(), 2);

        tv.push(Duration::from_nanos(2), ());
        assert_eq!(tv.checked_duration(), Some(Duration::from_nanos(1)));
        assert_eq!(tv.len(), 2);
    }

    #[test]
    fn push_above_the_non_min_limit() {
        let mut tv = TimeVec::<()>::builder()
            .with_limit_nanos(3)
            .build();
        
        tv.push(Duration::ZERO, ());
        tv.push(Duration::from_nanos(1), ());
        tv.push(Duration::from_nanos(2), ());
        tv.push(Duration::from_nanos(3), ());
        assert_eq!(tv.len(), 4);

        tv.push(Duration::from_nanos(4), ());
        assert_eq!(tv.len(), 4);
    }
}
