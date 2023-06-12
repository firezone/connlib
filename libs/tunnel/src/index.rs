use rand_core::{OsRng, RngCore};

// A basic linear-feedback shift register implemented as xorshift, used to
// distribute peer indexes across the 24-bit address space reserved for peer
// identification.
// The purpose is to obscure the total number of peers using the system and to
// ensure it requires a non-trivial amount of processing power and/or samples
// to guess other peers' indices. Anything more ambitious than this is wasted
// with only 24 bits of space.
pub(crate) struct IndexLfsr {
    initial: u32,
    lfsr: u32,
    mask: u32,
}

impl IndexLfsr {
    /// Generate a random 24-bit nonzero integer
    fn random_index() -> u32 {
        const LFSR_MAX: u32 = 0xffffff; // 24-bit seed
        loop {
            let i = OsRng.next_u32() & LFSR_MAX;
            if i > 0 {
                // LFSR seed must be non-zero
                break i;
            }
        }
    }

    /// Generate the next value in the pseudorandom sequence
    pub(crate) fn next(&mut self) -> u32 {
        // 24-bit polynomial for randomness. This is arbitrarily chosen to
        // inject bitflips into the value.
        const LFSR_POLY: u32 = 0xd80000; // 24-bit polynomial
        debug_assert_ne!(self.lfsr, 0);
        let value = self.lfsr - 1; // lfsr will never have value of 0
        self.lfsr = (self.lfsr >> 1) ^ ((0u32.wrapping_sub(self.lfsr & 1u32)) & LFSR_POLY);
        assert!(self.lfsr != self.initial, "Too many peers created");
        value ^ self.mask
    }
}

impl Default for IndexLfsr {
    fn default() -> Self {
        let seed = Self::random_index();
        IndexLfsr {
            initial: seed,
            lfsr: seed,
            mask: Self::random_index(),
        }
    }
}

// Checks that a packet has the index we expect
pub(crate) fn check_packet_index(recv_idx: u32, expected_idx: u32) -> bool {
    if (recv_idx >> 8) == expected_idx {
        true
    } else {
        tracing::warn!("receiver index doesn't match peer index, something fishy is going on");
        false
    }
}
