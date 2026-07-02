use std::io::{self, Read, Write};

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

fn main() -> io::Result<()> {
    let mut raw_buffer = Vec::new();
    io::stdin().read_to_end(&mut raw_buffer)?;

    let mut scanner = FastScanner::new(&raw_buffer);
    let n_val = match scanner.next_u32() {
        Some(val) => val,
        None => return Ok(()),
    };

    let mut writer = FastWriter::new(io::stdout().lock());

    // Use Algorithm A as the primary production solver for its zero-lookup stride layout
    if let Some((set_a, set_b)) = solve_block_stride(n_val) {
        writer.write_str("YES\n")?;
        
        // Print Set A
        writer.write_u32(set_a.len() as u32)?;
        writer.write_newline()?;
        let mut offset_idx = 0;
        while offset_idx < set_a.len() {
            if offset_idx > 0 {
                writer.write_space()?;
            }
            writer.write_u32(unsafe { *set_a.get_unchecked(offset_idx) })?;
            offset_idx += 1;
        }
        writer.write_newline()?;

        // Print Set B
        writer.write_u32(set_b.len() as u32)?;
        writer.write_newline()?;
        let mut offset_idx = 0;
        while offset_idx < set_b.len() {
            if offset_idx > 0 {
                writer.write_space()?;
            }
            writer.write_u32(unsafe { *set_b.get_unchecked(offset_idx) })?;
            offset_idx += 1;
        }
        writer.write_newline()?;
    } else {
        writer.write_str("NO\n")?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_solution(n: u32, set_a: &[u32], set_b: &[u32]) {
        let mut sum_a: u64 = 0;
        for &x in set_a {
            sum_a += x as u64;
        }
        let mut sum_b: u64 = 0;
        for &x in set_b {
            sum_b += x as u64;
        }
        assert_eq!(sum_a, sum_b, "Sums are not equal for n={}", n);
        
        let expected_sum = (n as u64) * (n as u64 + 1) / 4;
        assert_eq!(sum_a, expected_sum, "Sum is not equal to target sum for n={}", n);

        // Verify elements are 1..=n and distinct
        let mut present = vec![false; n as usize + 1];
        for &x in set_a {
            assert!(x >= 1 && x <= n);
            assert!(!present[x as usize]);
            present[x as usize] = true;
        }
        for &x in set_b {
            assert!(x >= 1 && x <= n);
            assert!(!present[x as usize]);
            present[x as usize] = true;
        }
        for i in 1..=n {
            assert!(present[i as usize], "Missing element {}", i);
        }
    }

    #[test]
    fn test_impossible_cases() {
        assert!(solve_block_stride(1).is_none());
        assert!(solve_block_stride(2).is_none());
        assert!(solve_block_stride(5).is_none());
        assert!(solve_block_stride(6).is_none());
        
        assert!(solve_greedy_vector(1).is_none());
        assert!(solve_greedy_vector(2).is_none());
        assert!(solve_greedy_vector(5).is_none());
        assert!(solve_greedy_vector(6).is_none());
    }

    #[test]
    fn test_valid_cases_block_stride() {
        let test_cases = [3, 4, 7, 8, 11, 12, 15, 16, 99, 100, 999, 1000];
        for &n in &test_cases {
            let res = solve_block_stride(n);
            assert!(res.is_some(), "Failed for n={}", n);
            let (set_a, set_b) = res.unwrap();
            verify_solution(n, &set_a, &set_b);
        }
    }

    #[test]
    fn test_valid_cases_greedy_vector() {
        let test_cases = [3, 4, 7, 8, 11, 12, 15, 16, 99, 100, 999, 1000];
        for &n in &test_cases {
            let res = solve_greedy_vector(n);
            assert!(res.is_some(), "Failed for n={}", n);
            let (set_a, set_b) = res.unwrap();
            verify_solution(n, &set_a, &set_b);
        }
    }
}
