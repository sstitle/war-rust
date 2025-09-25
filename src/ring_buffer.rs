/// A fixed-size ring buffer implementation using stack allocation
/// Generic over type T and size N for compile-time size guarantees
#[derive(Debug, Clone)]
pub struct RingBuffer<T: Copy, const N: usize> {
    buffer: [T; N],
    head: usize,  // Points to the next position to write
    tail: usize,  // Points to the next position to read
    count: usize, // Number of elements currently in buffer
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    /// Create a new empty ring buffer with a default value for initialization
    pub fn new(default_value: T) -> Self {
        Self {
            buffer: [default_value; N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    /// Returns the number of elements in the buffer
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns true if the buffer is full
    pub fn is_full(&self) -> bool {
        self.count == N
    }

    /// Returns the maximum capacity of the buffer
    pub fn capacity(&self) -> usize {
        N
    }

    /// Push an element to the back of the buffer
    /// Returns true if successful, false if buffer is full
    pub fn push_back(&mut self, item: T) -> bool {
        if self.is_full() {
            return false;
        }

        self.buffer[self.head] = item;
        self.head = (self.head + 1) % N;
        self.count += 1;
        true
    }

    /// Pop an element from the front of the buffer
    /// Returns Some(T) if successful, None if buffer is empty
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = self.buffer[self.tail];
        self.tail = (self.tail + 1) % N;
        self.count -= 1;
        Some(item)
    }

    /// Push an element to the front of the buffer (prepend)
    /// Returns true if successful, false if buffer is full
    pub fn push_front(&mut self, item: T) -> bool {
        if self.is_full() {
            return false;
        }

        self.tail = if self.tail == 0 { N - 1 } else { self.tail - 1 };
        self.buffer[self.tail] = item;
        self.count += 1;
        true
    }

    /// Pop an element from the back of the buffer
    /// Returns Some(T) if successful, None if buffer is empty
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.head = if self.head == 0 { N - 1 } else { self.head - 1 };
        let item = self.buffer[self.head];
        self.count -= 1;
        Some(item)
    }

    /// Add multiple items to the front of the buffer (useful for winning cards in War)
    /// Items are added in reverse order so the first item in the slice becomes the front
    /// Returns the number of items successfully added
    pub fn push_front_multiple(&mut self, items: &[T]) -> usize {
        let mut added = 0;
        for &item in items.iter().rev() {
            if self.push_front(item) {
                added += 1;
            } else {
                break; // Buffer is full
            }
        }
        added
    }

    /// Add multiple items to the back of the buffer
    /// Returns the number of items successfully added
    pub fn push_back_multiple(&mut self, items: &[T]) -> usize {
        let mut added = 0;
        for &item in items.iter() {
            if self.push_back(item) {
                added += 1;
            } else {
                break; // Buffer is full
            }
        }
        added
    }

    /// Peek at the front element without removing it
    pub fn front(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.buffer[self.tail])
        }
    }

    /// Peek at the back element without removing it
    pub fn back(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let back_idx = if self.head == 0 { N - 1 } else { self.head - 1 };
            Some(self.buffer[back_idx])
        }
    }

    /// Clear all elements from the buffer
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }

    /// Create an iterator over the elements in order (front to back)
    pub fn iter(&self) -> RingBufferIter<T, N> {
        RingBufferIter {
            buffer: self,
            current: self.tail,
            remaining: self.count,
        }
    }
}

/// Iterator for RingBuffer
pub struct RingBufferIter<'a, T: Copy, const N: usize> {
    buffer: &'a RingBuffer<T, N>,
    current: usize,
    remaining: usize,
}

impl<'a, T: Copy, const N: usize> Iterator for RingBufferIter<'a, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let item = self.buffer.buffer[self.current];
        self.current = (self.current + 1) % N;
        self.remaining -= 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a, T: Copy, const N: usize> ExactSizeIterator for RingBufferIter<'a, T, N> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut rb = RingBuffer::<i32, 5>::new(0);

        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.capacity(), 5);

        // Test push_back and pop_front
        assert!(rb.push_back(1));
        assert!(rb.push_back(2));
        assert!(rb.push_back(3));
        assert_eq!(rb.len(), 3);

        assert_eq!(rb.pop_front(), Some(1));
        assert_eq!(rb.pop_front(), Some(2));
        assert_eq!(rb.len(), 1);

        assert_eq!(rb.pop_front(), Some(3));
        assert!(rb.is_empty());
        assert_eq!(rb.pop_front(), None);
    }

    #[test]
    fn test_front_operations() {
        let mut rb = RingBuffer::<i32, 4>::new(0);

        assert!(rb.push_front(1));
        assert!(rb.push_front(2));
        assert_eq!(rb.len(), 2);

        assert_eq!(rb.pop_front(), Some(2));
        assert_eq!(rb.pop_front(), Some(1));
        assert!(rb.is_empty());
    }

    #[test]
    fn test_wraparound() {
        let mut rb = RingBuffer::<i32, 3>::new(0);

        // Fill the buffer
        assert!(rb.push_back(1));
        assert!(rb.push_back(2));
        assert!(rb.push_back(3));
        assert!(rb.is_full());
        assert!(!rb.push_back(4)); // Should fail

        // Remove and add to test wraparound
        assert_eq!(rb.pop_front(), Some(1));
        assert!(rb.push_back(4));
        assert!(rb.is_full());

        assert_eq!(rb.pop_front(), Some(2));
        assert_eq!(rb.pop_front(), Some(3));
        assert_eq!(rb.pop_front(), Some(4));
        assert!(rb.is_empty());
    }

    #[test]
    fn test_multiple_operations() {
        let mut rb = RingBuffer::<i32, 10>::new(0);

        let items = vec![1, 2, 3, 4, 5];
        assert_eq!(rb.push_back_multiple(&items), 5);
        assert_eq!(rb.len(), 5);

        let front_items = vec![10, 20];
        assert_eq!(rb.push_front_multiple(&front_items), 2);
        assert_eq!(rb.len(), 7);

        // Should be: [10, 20, 1, 2, 3, 4, 5] (reversed order due to push_front_multiple)
        assert_eq!(rb.pop_front(), Some(10));
        assert_eq!(rb.pop_front(), Some(20));
        assert_eq!(rb.pop_front(), Some(1));
    }
}
