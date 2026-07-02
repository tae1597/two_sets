use std::io::{self, Write};

/// Fast scanner for parsing integers directly from a raw byte buffer.
/// Bypasses high-level string allocations and regex parsing.
pub struct FastScanner<'a> {
    raw_buffer: &'a [u8],
    byte_cursor: usize,
}

impl<'a> FastScanner<'a> {
    #[inline(always)]
    pub fn new(raw_buffer: &'a [u8]) -> Self {
        Self {
            raw_buffer,
            byte_cursor: 0,
        }
    }

    /// Parses the next u32 integer from the byte stream.
    #[inline(always)]
    pub fn next_u32(&mut self) -> Option<u32> {
        // Skip leading whitespace bytes (ASCII <= 32)
        while self.byte_cursor < self.raw_buffer.len() 
            && unsafe { *self.raw_buffer.get_unchecked(self.byte_cursor) } <= b' ' 
        {
            self.byte_cursor += 1;
        }

        if self.byte_cursor >= self.raw_buffer.len() {
            return None;
        }

        let mut val_lookup: u32 = 0;
        // Parse digits sequentially
        while self.byte_cursor < self.raw_buffer.len() {
            let byte = unsafe { *self.raw_buffer.get_unchecked(self.byte_cursor) };
            if byte > b' ' {
                val_lookup = val_lookup * 10 + (byte - b'0') as u32;
                self.byte_cursor += 1;
            } else {
                break;
            }
        }
        Some(val_lookup)
    }
}

/// Fast writer that formats integers directly to a buffered byte stream.
/// Bypasses core::fmt parsing overhead and minimizes write system calls.
pub struct FastWriter<W: Write> {
    writer: io::BufWriter<W>,
    temp_buf: [u8; 20],
}

impl<W: Write> FastWriter<W> {
    #[inline(always)]
    pub fn new(inner: W) -> Self {
        Self {
            writer: io::BufWriter::with_capacity(128 * 1024, inner),
            temp_buf: [0; 20],
        }
    }

    #[inline(always)]
    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())
    }

    #[inline(always)]
    pub fn write_u32(&mut self, mut val: u32) -> io::Result<()> {
        if val == 0 {
            self.writer.write_all(b"0")?;
            return Ok(());
        }
        let mut byte_cursor = 20;
        while val > 0 {
            byte_cursor -= 1;
            self.temp_buf[byte_cursor] = b'0' + (val % 10) as u8;
            val /= 10;
        }
        self.writer.write_all(&self.temp_buf[byte_cursor..])
    }

    #[inline(always)]
    pub fn write_space(&mut self) -> io::Result<()> {
        self.writer.write_all(b" ")
    }

    #[inline(always)]
    pub fn write_newline(&mut self) -> io::Result<()> {
        self.writer.write_all(b"\n")
    }

    #[inline(always)]
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Algorithm A: Zero-Allocation Stride-Based Block Partitioning.
/// This method leverages mathematical properties of n % 4 to divide numbers
/// in strides of 4, ensuring O(n) time and O(n) space (to store the results).
#[inline(never)]
pub fn solve_block_stride(n_val: u32) -> Option<(Vec<u32>, Vec<u32>)> {
    let remainder = n_val % 4;
    if remainder == 1 || remainder == 2 {
        return None;
    }

    let mut set_a = Vec::with_capacity((n_val / 2) as usize + 1);
    let mut set_b = Vec::with_capacity((n_val / 2) as usize + 1);

    if remainder == 0 {
        // Stride pattern of size 4: (4k+1, 4k+2, 4k+3, 4k+4)
        // Group sum: (4k+1) + (4k+4) = (4k+2) + (4k+3) = 8k+5
        let stride_length = 4;
        let mut offset_idx = 1;
        while offset_idx <= n_val {
            set_a.push(offset_idx);
            set_b.push(offset_idx + 1);
            set_b.push(offset_idx + 2);
            set_a.push(offset_idx + 3);
            offset_idx += stride_length;
        }
    } else {
        // remainder == 3
        // Base case for first 3 elements: {1, 2} vs {3}
        set_a.push(1);
        set_a.push(2);
        set_b.push(3);

        // Stride pattern starting from 4: (4k, 4k+1, 4k+2, 4k+3) for k >= 1
        // Group sum: 4k + (4k+3) = (4k+1) + (4k+2) = 8k+3
        let stride_length = 4;
        let mut offset_idx = 4;
        while offset_idx <= n_val {
            set_a.push(offset_idx);
            set_b.push(offset_idx + 1);
            set_b.push(offset_idx + 2);
            set_a.push(offset_idx + 3);
            offset_idx += stride_length;
        }
    }

    Some((set_a, set_b))
}

/// Algorithm B: State-Based Greedy Boolean Vector Partitioning.
/// This method uses a contiguous flat u8 buffer to represent set membership.
/// It iterates backwards from n down to 1, greedily placing elements into Set 1
/// if they fit within the remaining target sum, and then gathers elements by
/// scanning the buffer.
#[inline(never)]
pub fn solve_greedy_vector(n_val: u32) -> Option<(Vec<u32>, Vec<u32>)> {
    let remainder = n_val % 4;
    if remainder == 1 || remainder == 2 {
        return None;
    }

    let n_usize = n_val as usize;
    let mut membership_buffer = vec![0u8; n_usize + 1];
    
    // total_sum / 2 = n * (n + 1) / 4
    let mut target_sum: u64 = (n_val as u64) * ((n_val as u64) + 1) / 4;
    
    // Greedy loop from n down to 1
    let mut val_lookup = n_val;
    while val_lookup >= 1 {
        let val_u64 = val_lookup as u64;
        if val_u64 <= target_sum {
            unsafe {
                *membership_buffer.get_unchecked_mut(val_lookup as usize) = 1;
            }
            target_sum -= val_u64;
        }
        val_lookup -= 1;
    }

    let mut set_a = Vec::with_capacity((n_val / 2) as usize + 1);
    let mut set_b = Vec::with_capacity((n_val / 2) as usize + 1);

    // Sequential scan over membership_buffer leverages CPU cache line prefetching (64-byte blocks)
    let mut scan_idx = 1;
    while scan_idx <= n_usize {
        let is_in_set1 = unsafe { *membership_buffer.get_unchecked(scan_idx) };
        if is_in_set1 == 1 {
            set_a.push(scan_idx as u32);
        } else {
            set_b.push(scan_idx as u32);
        }
        scan_idx += 1;
    }

    Some((set_a, set_b))
}
