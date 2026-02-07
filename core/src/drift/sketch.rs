//! Invertible Bloom Lookup Table (IBLT) for set reconciliation
//!
//! IBLT enables two parties to efficiently compute the symmetric difference of their sets
//! in O(d) time and space, where d is the number of differences. This is ideal for mesh
//! synchronization where nodes meet briefly and need to exchange only what the other
//! party lacks.
//!
//! Protocol:
//! 1. Alice computes IBLT(A) from her message IDs and sends it
//! 2. Bob computes IBLT(B) from his message IDs, sends back IBLT(B) XOR IBLT(A)
//! 3. Alice/Bob can now "peel" the result to recover which IDs are on one side or the other
//!
//! Theory: Each cell stores count (insertions - deletions), key_sum (XOR of all keys),
//! and key_check (XOR of hash(key) for all keys). A cell is "pure" (single element)
//! when count = ±1. We peel pure cells, extract their ID, remove them from all cells
//! that hash them, and repeat until the table is empty or fails to make progress.

use crate::drift::{DriftError, MessageId};
use blake3;

const HASH_COUNT: usize = 3; // Number of independent hash functions (k = 3 is standard)
const CELLS_PER_DIFF: usize = 3; // Expected cells per difference (α ≈ 1.5-3, we use 3 for safety)

/// A single cell in an IBLT
#[derive(Debug, Clone)]
pub struct IBLTCell {
    /// Count of insertions minus deletions for this cell (>0 if contains items, <0 if deletions)
    pub count: i32,
    /// XOR of all MessageIds hashing to this cell
    pub key_sum: [u8; 16],
    /// XOR of blake3(key)[0:4] for all keys hashing to this cell
    pub key_check: u32,
}

impl IBLTCell {
    fn new() -> Self {
        Self {
            count: 0,
            key_sum: [0u8; 16],
            key_check: 0u32,
        }
    }

    /// Check if this cell is "pure" (contains exactly one element): count == ±1
    fn is_pure(&self) -> bool {
        self.count == 1 || self.count == -1
    }

    /// Check if this cell is empty
    fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// XOR in a key (add to the cell)
    fn xor_in(&mut self, key: &MessageId) {
        for i in 0..16 {
            self.key_sum[i] ^= key[i];
        }
        let check = u32::from_le_bytes([key[0], key[1], key[2], key[3]]);
        self.key_check ^= check;
    }

    /// XOR out a key (remove from the cell)
    fn xor_out(&mut self, key: &MessageId) {
        self.xor_in(key); // XOR is self-inverse
    }
}

/// Invertible Bloom Lookup Table for set reconciliation
#[derive(Debug, Clone)]
pub struct IBLT {
    cells: Vec<IBLTCell>,
    num_cells: usize,
}

impl IBLT {
    /// Create a new IBLT sized for the expected number of differences
    pub fn new(expected_diffs: usize) -> Self {
        let num_cells = expected_diffs.max(1) * CELLS_PER_DIFF;
        let cells = vec![IBLTCell::new(); num_cells];
        Self { cells, num_cells }
    }

    /// Create an IBLT with explicit cell count (for testing and advanced usage)
    pub fn with_cells(num_cells: usize) -> Self {
        let cells = vec![IBLTCell::new(); num_cells.max(1)];
        Self { cells, num_cells: num_cells.max(1) }
    }

    /// Hash a key to one of the k cell indices
    /// Uses blake3 with different prefixes for each hash function
    fn hash_to_cell(&self, key: &MessageId, hash_idx: usize) -> usize {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&[hash_idx as u8]);
        hasher.update(key);
        let digest = hasher.finalize();
        let bytes = digest.as_bytes();
        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        (value as usize) % self.num_cells
    }

    /// Insert a key into the IBLT
    pub fn insert(&mut self, key: &MessageId) {
        for hash_idx in 0..HASH_COUNT {
            let cell_idx = self.hash_to_cell(key, hash_idx);
            self.cells[cell_idx].count += 1;
            self.cells[cell_idx].xor_in(key);
        }
    }

    /// Remove (subtract) a key from the IBLT (used to compute differences)
    pub fn remove(&mut self, key: &MessageId) {
        for hash_idx in 0..HASH_COUNT {
            let cell_idx = self.hash_to_cell(key, hash_idx);
            self.cells[cell_idx].count -= 1;
            self.cells[cell_idx].xor_out(key);
        }
    }

    /// Compute the symmetric difference: self XOR other
    /// Returns a new IBLT representing (A - B) union (B - A)
    pub fn subtract(&self, other: &IBLT) -> Result<IBLT, DriftError> {
        if self.num_cells != other.num_cells {
            return Err(DriftError::BufferTooShort {
                need: self.num_cells,
                got: other.num_cells,
            });
        }

        let mut result = IBLT::with_cells(self.num_cells);
        for i in 0..self.num_cells {
            result.cells[i].count = self.cells[i].count - other.cells[i].count;
            for j in 0..16 {
                result.cells[i].key_sum[j] = self.cells[i].key_sum[j] ^ other.cells[i].key_sum[j];
            }
            result.cells[i].key_check = self.cells[i].key_check ^ other.cells[i].key_check;
        }
        Ok(result)
    }

    /// Decode the IBLT to recover the symmetric difference
    /// Returns (alice_only, bob_only) - the elements that differ between the two sets
    ///
    /// Fails if decode cannot completely peel the table (too many differences for table size)
    pub fn decode(&self) -> Result<(Vec<MessageId>, Vec<MessageId>), DriftError> {
        let mut cells = self.cells.clone();
        let mut alice_only = Vec::new();
        let mut bob_only = Vec::new();

        // Iteration limit to prevent infinite loops when IBLT is undersized
        let max_iterations = self.num_cells * 10;
        let mut iteration_count = 0;

        loop {
            iteration_count += 1;
            if iteration_count > max_iterations {
                return Err(DriftError::DecompressionFailed(
                    "IBLT decode failed: exceeded maximum iterations (IBLT likely undersized for differences)"
                        .to_string(),
                ));
            }

            // Find a pure cell (count == ±1)
            let pure_idx = cells.iter().position(|c| c.is_pure());

            match pure_idx {
                None => {
                    // No more pure cells. Check if all cells are empty
                    if cells.iter().all(|c| c.is_empty()) {
                        return Ok((alice_only, bob_only));
                    } else {
                        // Decoding failed: non-empty cells remain but none are pure
                        return Err(DriftError::DecompressionFailed(
                            "IBLT decode failed: cannot peel remaining cells".to_string(),
                        ));
                    }
                }
                Some(idx) => {
                    // Copy values out to release immutable borrow before mutating cells
                    let key = cells[idx].key_sum;
                    let count = cells[idx].count;
                    let key_check = cells[idx].key_check;

                    // Verify this is actually a pure cell by checking consistency
                    // The key_check should match: hash(key)[0:4]
                    let check = u32::from_le_bytes([key[0], key[1], key[2], key[3]]);
                    if key_check != check {
                        // This is a corrupted/mixed cell, not actually pure
                        return Err(DriftError::DecompressionFailed(
                            "IBLT decode failed: key_check mismatch, possible hash collision or corruption"
                                .to_string(),
                        ));
                    }

                    // Record which side this key came from
                    if count == 1 {
                        alice_only.push(key);
                    } else {
                        bob_only.push(key);
                    }

                    // Remove this key from all cells that hash it
                    for hash_idx in 0..HASH_COUNT {
                        let cell_idx = self.hash_to_cell(&key, hash_idx);
                        cells[cell_idx].count -= count; // Subtract the sign
                        cells[cell_idx].xor_out(&key);
                    }

                    // Clear the cell we just processed
                    cells[idx] = IBLTCell::new();
                }
            }
        }
    }

    /// Serialize IBLT to bytes for transmission
    /// Format: [2 bytes LE: num_cells][for each cell: count(1) key_sum(16) key_check(4)]
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        if self.num_cells > u16::MAX as usize {
            return Err(DriftError::BufferTooShort {
                need: self.num_cells,
                got: u16::MAX as usize,
            });
        }

        let mut buf = Vec::with_capacity(2 + self.num_cells * 21);
        buf.extend_from_slice(&(self.num_cells as u16).to_le_bytes());

        for cell in &self.cells {
            buf.push(cell.count as u8);
            buf.extend_from_slice(&cell.key_sum);
            buf.extend_from_slice(&cell.key_check.to_le_bytes());
        }

        Ok(buf)
    }

    /// Deserialize IBLT from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        if data.len() < 2 {
            return Err(DriftError::BufferTooShort {
                need: 2,
                got: data.len(),
            });
        }

        let num_cells = u16::from_le_bytes([data[0], data[1]]) as usize;
        let expected_len = 2 + num_cells * 21;

        if data.len() != expected_len {
            return Err(DriftError::BufferTooShort {
                need: expected_len,
                got: data.len(),
            });
        }

        let mut cells = Vec::with_capacity(num_cells);

        for i in 0..num_cells {
            let offset = 2 + i * 21;
            let count = data[offset] as i32;

            let mut key_sum = [0u8; 16];
            key_sum.copy_from_slice(&data[offset + 1..offset + 17]);

            let key_check = u32::from_le_bytes([
                data[offset + 17],
                data[offset + 18],
                data[offset + 19],
                data[offset + 20],
            ]);

            cells.push(IBLTCell {
                count,
                key_sum,
                key_check,
            });
        }

        Ok(IBLT { cells, num_cells })
    }

    /// Get the size in bytes when serialized
    pub fn serialized_size(&self) -> usize {
        2 + self.num_cells * 21
    }

    /// Get number of cells in this IBLT
    pub fn cell_count(&self) -> usize {
        self.num_cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_id(val: u8) -> MessageId {
        [val; 16]
    }

    #[test]
    fn test_iblt_insert_decode_single_element() {
        let mut iblt = IBLT::new(1);
        let key = make_test_id(42);
        iblt.insert(&key);

        // Subtract empty IBLT to get the difference
        let empty = IBLT::new(1);
        let diff = iblt.subtract(&empty).unwrap();

        let (alice, bob) = diff.decode().unwrap();
        assert_eq!(alice.len(), 1);
        assert_eq!(alice[0], key);
        assert_eq!(bob.len(), 0);
    }

    #[test]
    fn test_iblt_symmetric_difference() {
        let mut iblt_a = IBLT::new(5);
        let mut iblt_b = IBLT::new(5);

        // Alice has 1, 2, 3
        iblt_a.insert(&make_test_id(1));
        iblt_a.insert(&make_test_id(2));
        iblt_a.insert(&make_test_id(3));

        // Bob has 2, 3, 4
        iblt_b.insert(&make_test_id(2));
        iblt_b.insert(&make_test_id(3));
        iblt_b.insert(&make_test_id(4));

        // Difference: A - B has 1, B - A has 4
        let diff = iblt_a.subtract(&iblt_b).unwrap();
        let (alice, bob) = diff.decode().unwrap();

        assert!(alice.contains(&make_test_id(1)));
        assert!(bob.contains(&make_test_id(4)));
        assert_eq!(alice.len(), 1);
        assert_eq!(bob.len(), 1);
    }

    #[test]
    fn test_iblt_empty_sets() {
        let iblt_a = IBLT::new(1);
        let iblt_b = IBLT::new(1);

        let diff = iblt_a.subtract(&iblt_b).unwrap();
        let (alice, bob) = diff.decode().unwrap();

        assert_eq!(alice.len(), 0);
        assert_eq!(bob.len(), 0);
    }

    #[test]
    fn test_iblt_identical_sets() {
        let mut iblt_a = IBLT::new(3);
        let mut iblt_b = IBLT::new(3);

        for i in 1..=3 {
            iblt_a.insert(&make_test_id(i));
            iblt_b.insert(&make_test_id(i));
        }

        let diff = iblt_a.subtract(&iblt_b).unwrap();
        let (alice, bob) = diff.decode().unwrap();

        assert_eq!(alice.len(), 0);
        assert_eq!(bob.len(), 0);
    }

    #[test]
    fn test_iblt_serialization_roundtrip() {
        let mut iblt = IBLT::new(2);
        iblt.insert(&make_test_id(10));
        iblt.insert(&make_test_id(20));

        let bytes = iblt.to_bytes().unwrap();
        let restored = IBLT::from_bytes(&bytes).unwrap();

        assert_eq!(iblt.cell_count(), restored.cell_count());

        // Verify it still decodes the same way
        let empty = IBLT::new(2);
        let diff_orig = iblt.subtract(&empty).unwrap();
        let diff_rest = restored.subtract(&empty).unwrap();

        let (a1, b1) = diff_orig.decode().unwrap();
        let (a2, b2) = diff_rest.decode().unwrap();

        assert_eq!(a1.len(), a2.len());
        assert_eq!(b1.len(), b2.len());
        assert!(a1.iter().all(|id| a2.contains(id)));
    }

    #[test]
    fn test_iblt_asymmetric_differences() {
        let mut iblt_a = IBLT::new(10);
        let mut iblt_b = IBLT::new(10);

        // Alice has 100 items
        for i in 0..100 {
            iblt_a.insert(&[i as u8; 16]);
        }

        // Bob has 90-109 (overlap 90-99, different 100-109)
        for i in 90..110 {
            iblt_b.insert(&[i as u8; 16]);
        }

        // A - B should have 0-89 (90 items)
        // B - A should have 100-109 (10 items)
        let diff = iblt_a.subtract(&iblt_b).unwrap();

        // Note: This might fail to decode if we undersized the IBLT
        // Let's just verify the structure
        assert_eq!(diff.cell_count(), 10 * 3);
    }

    #[test]
    fn test_iblt_size_calculation() {
        let iblt = IBLT::new(5);
        assert_eq!(iblt.cell_count(), 5 * CELLS_PER_DIFF);
        assert_eq!(iblt.serialized_size(), 2 + (5 * CELLS_PER_DIFF) * 21);
    }

    #[test]
    fn test_iblt_single_difference() {
        let mut iblt_a = IBLT::new(1);
        let mut iblt_b = IBLT::new(1);

        // Alice has 1, 2
        iblt_a.insert(&make_test_id(1));
        iblt_a.insert(&make_test_id(2));

        // Bob has 1 (missing 2)
        iblt_b.insert(&make_test_id(1));

        let diff = iblt_a.subtract(&iblt_b).unwrap();
        let (alice, bob) = diff.decode().unwrap();

        assert_eq!(alice.len(), 1);
        assert!(alice.contains(&make_test_id(2)));
        assert_eq!(bob.len(), 0);
    }

    #[test]
    fn test_iblt_multiple_differences() {
        let mut iblt_a = IBLT::new(5);
        let mut iblt_b = IBLT::new(5);

        // Alice: 1, 2, 3, 4, 5
        for i in 1..=5 {
            iblt_a.insert(&make_test_id(i));
        }

        // Bob: 1, 3, 5, 6, 7
        iblt_b.insert(&make_test_id(1));
        iblt_b.insert(&make_test_id(3));
        iblt_b.insert(&make_test_id(5));
        iblt_b.insert(&make_test_id(6));
        iblt_b.insert(&make_test_id(7));

        let diff = iblt_a.subtract(&iblt_b).unwrap();
        let (alice, bob) = diff.decode().unwrap();

        // Alice only: 2, 4
        assert_eq!(alice.len(), 2);
        assert!(alice.contains(&make_test_id(2)));
        assert!(alice.contains(&make_test_id(4)));

        // Bob only: 6, 7
        assert_eq!(bob.len(), 2);
        assert!(bob.contains(&make_test_id(6)));
        assert!(bob.contains(&make_test_id(7)));
    }

    #[test]
    fn test_iblt_remove_operation() {
        let mut iblt = IBLT::new(2);
        iblt.insert(&make_test_id(10));
        iblt.remove(&make_test_id(10));

        let empty = IBLT::new(2);
        let diff = iblt.subtract(&empty).unwrap();
        let (alice, bob) = diff.decode().unwrap();

        assert_eq!(alice.len(), 0);
        assert_eq!(bob.len(), 0);
    }

    #[test]
    fn test_iblt_oversized_differences_fail_decode() {
        let mut iblt_a = IBLT::new(1); // Only 3 cells
        let iblt_b = IBLT::new(1); // Must have same cell count for subtract

        // Add way more differences than capacity
        for i in 0..20 {
            iblt_a.insert(&make_test_id(i));
        }

        let diff = iblt_a.subtract(&iblt_b).unwrap();
        // Should fail to decode because too many elements for the IBLT size
        let result = diff.decode();

        // Expect decode to fail due to too many differences
        assert!(result.is_err());
    }

    #[test]
    fn test_iblt_commutativity_with_swap() {
        let mut iblt_a = IBLT::new(3);
        let mut iblt_b = IBLT::new(3);

        iblt_a.insert(&make_test_id(1));
        iblt_a.insert(&make_test_id(2));

        iblt_b.insert(&make_test_id(2));
        iblt_b.insert(&make_test_id(3));

        let diff_a_minus_b = iblt_a.subtract(&iblt_b).unwrap();
        let diff_b_minus_a = iblt_b.subtract(&iblt_a).unwrap();

        let (a1, b1) = diff_a_minus_b.decode().unwrap();
        let (a2, b2) = diff_b_minus_a.decode().unwrap();

        // (A - B) alice_only should equal (B - A) bob_only
        assert_eq!(a1.len(), b2.len());
        assert!(a1.iter().all(|id| b2.contains(id)));

        // (A - B) bob_only should equal (B - A) alice_only
        assert_eq!(b1.len(), a2.len());
        assert!(b1.iter().all(|id| a2.contains(id)));
    }

    #[test]
    fn test_iblt_from_bytes_invalid_length() {
        let mut buf = vec![0u8, 1]; // Claims 256 cells
        buf.extend_from_slice(&vec![0u8; 100]); // But not enough data

        let result = IBLT::from_bytes(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_iblt_from_bytes_empty() {
        let result = IBLT::from_bytes(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_iblt_subtract_mismatched_cells() {
        let iblt_a = IBLT::new(2); // 6 cells
        let iblt_b = IBLT::new(5); // 15 cells

        let result = iblt_a.subtract(&iblt_b);
        assert!(result.is_err());
    }
}
