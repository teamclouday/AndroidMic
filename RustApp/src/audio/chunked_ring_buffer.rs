use std::{cmp::max, fmt::Debug};

#[derive(Clone)]
pub struct ChunkedRingBuffer<T> {
    v: Vec<T>,
    cap: usize,
    chunk_size: usize,
    /// Index of the first element of the first chunk
    ptr_start: usize,
    /// Index where the next element will be inserted
    ptr_end: usize,
    /// Distinguishes between empty and full states when ptr_start == ptr_end   
    is_full: bool,
}

impl<T> Debug for ChunkedRingBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkedRingBuffer")
            .field("cap", &self.cap)
            .field("chunk_size", &self.chunk_size)
            .field("ptr_start", &self.ptr_start)
            .field("ptr_end", &self.ptr_end)
            .field("full", &self.is_full)
            .finish()
    }
}

impl<T> ChunkedRingBuffer<T> {
    pub fn new(nb_chunk: usize, chunk_size: usize) -> Self {
        let cap = nb_chunk * chunk_size;
        let mut v = Vec::with_capacity(cap);
        #[allow(clippy::uninit_vec)]
        unsafe {
            v.set_len(cap)
        };
        Self {
            v,
            cap,
            chunk_size,
            ptr_start: 0,
            ptr_end: 0,
            is_full: false,
        }
    }

    fn grow(&mut self, min_cap: usize)
    where
        T: Copy,
    {
        let old_len = self.len();
        let old_cap = self.cap;

        let min_cap = min_cap.div_ceil(self.chunk_size) * self.chunk_size;
        let new_cap = max(self.cap * 2, min_cap);

        let mut new_v = Vec::with_capacity(new_cap);
        #[allow(clippy::uninit_vec)]
        unsafe {
            new_v.set_len(new_cap)
        };

        if old_len > 0 {
            if self.ptr_start < self.ptr_end {
                new_v[0..old_len].copy_from_slice(&self.v[self.ptr_start..self.ptr_end]);
            } else {
                let part1_len = old_cap - self.ptr_start;
                new_v[0..part1_len].copy_from_slice(&self.v[self.ptr_start..old_cap]);
                new_v[part1_len..part1_len + self.ptr_end]
                    .copy_from_slice(&self.v[0..self.ptr_end]);
            }
        }

        self.ptr_start = 0;
        self.ptr_end = old_len;
        self.cap = new_cap;
        self.v = new_v;
        self.is_full = false;
    }

    pub fn len(&self) -> usize {
        if self.is_full {
            self.cap
        } else if self.ptr_start <= self.ptr_end {
            self.ptr_end - self.ptr_start
        } else {
            (self.cap - self.ptr_start) + self.ptr_end
        }
    }

    pub fn extend(&mut self, slice: &[T])
    where
        T: Copy,
    {
        if slice.is_empty() {
            return;
        }

        if self.cap < self.len() + slice.len() {
            self.grow(self.len() + slice.len());
        }

        let slice_len = slice.len();

        let mut copied = 0;

        if self.ptr_start <= self.ptr_end {
            let to_be_copied = (self.cap - self.ptr_end).min(slice_len);
            self.v[self.ptr_end..self.ptr_end + to_be_copied]
                .copy_from_slice(&slice[0..to_be_copied]);
            copied += to_be_copied;

            if copied < slice_len {
                let to_be_copied = (self.ptr_start).min(slice_len - copied);
                self.v[0..to_be_copied].copy_from_slice(&slice[copied..copied + to_be_copied]);
                copied += to_be_copied;
            }
        } else {
            self.v[self.ptr_end..self.ptr_end + slice_len].copy_from_slice(slice);
            copied = slice_len;
        }

        self.ptr_end = (self.ptr_end + copied) % self.cap;
        self.is_full = self.ptr_start == self.ptr_end;
    }

    pub fn first_chunk(&self) -> &[T] {
        debug_assert!(self.has_chunk_available());
        &self.v[self.ptr_start..self.ptr_start + self.chunk_size]
    }

    pub fn first_chunk_mut(&mut self) -> &mut [T] {
        debug_assert!(self.has_chunk_available());
        &mut self.v[self.ptr_start..self.ptr_start + self.chunk_size]
    }

    pub fn remove_first_chunk(&mut self) {
        debug_assert!(self.has_chunk_available());
        self.remove_n(self.chunk_size);
    }

    fn remove_n(&mut self, n: usize) {
        debug_assert!(
            self.len() >= n,
            "Attempted to remove {} elements, but only {} are available",
            n,
            self.len(),
        );

        self.ptr_start = (self.ptr_start + n) % self.cap;
        self.is_full = false;
    }

    pub fn number_of_chunk(&self) -> usize {
        self.len() / self.chunk_size
    }

    pub fn has_chunk_available(&self) -> bool {
        self.len() >= self.chunk_size
    }
}
