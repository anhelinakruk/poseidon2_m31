//! Poseidon2 sponge construction for hashing.

use crate::poseidon2::{poseidon2_permutation, N_STATE, RATE};
use stwo::core::fields::m31::BaseField;

/// Poseidon2 sponge hasher.
///
/// Absorbs elements in blocks of RATE (8), automatically pads with zeros.
#[derive(Clone)]
pub struct Poseidon2Sponge {
    state: [BaseField; N_STATE],
    buffer: Vec<BaseField>,
}

impl Poseidon2Sponge {
    /// Creates a new hasher with zero state.
    pub fn new() -> Self {
        Self {
            state: [BaseField::from_u32_unchecked(0); N_STATE],
            buffer: Vec::new(),
        }
    }

    /// Absorbs a single field element.
    pub fn absorb(&mut self, element: BaseField) {
        self.buffer.push(element);
        if self.buffer.len() == RATE {
            self.process_block();
        }
    }

    /// Absorbs multiple field elements.
    pub fn absorb_many(&mut self, elements: &[BaseField]) {
        for &element in elements {
            self.absorb(element);
        }
    }

    fn process_block(&mut self) {
        for i in 0..RATE {
            self.state[i] += self.buffer[i];
        }
        poseidon2_permutation(&mut self.state);
        self.buffer.clear();
    }

    /// Finalizes and returns the hash (first state element).
    /// Automatically pads with zeros if needed.
    pub fn finalize(mut self) -> BaseField {
        if !self.buffer.is_empty() {
            while self.buffer.len() < RATE {
                self.buffer.push(BaseField::from_u32_unchecked(0));
            }
            self.process_block();
        }
        self.state[0]
    }

    /// Finalizes and returns the rate portion (8 elements).
    pub fn finalize_full_rate(mut self) -> [BaseField; RATE] {
        if !self.buffer.is_empty() {
            while self.buffer.len() < RATE {
                self.buffer.push(BaseField::from_u32_unchecked(0));
            }
            self.process_block();
        }
        std::array::from_fn(|i| self.state[i])
    }

    /// Finalizes and returns the full state (16 elements).
    pub fn finalize_full_state(mut self) -> [BaseField; N_STATE] {
        if !self.buffer.is_empty() {
            while self.buffer.len() < RATE {
                self.buffer.push(BaseField::from_u32_unchecked(0));
            }
            self.process_block();
        }
        self.state
    }
}

impl Default for Poseidon2Sponge {
    fn default() -> Self {
        Self::new()
    }
}

/// Hashes a slice of field elements.
///
/// Automatically pads with zeros to multiples of RATE (8).
pub fn hash(elements: &[BaseField]) -> BaseField {
    let mut sponge = Poseidon2Sponge::new();
    sponge.absorb_many(elements);
    sponge.finalize()
}

/// Hashes multiple messages with vertical chaining.
///
/// Each message is RATE (8) elements. Output state chains to next message.
pub fn hash_messages(messages: &[[BaseField; RATE]]) -> Vec<[BaseField; N_STATE]> {
    let mut outputs = Vec::with_capacity(messages.len());
    let mut prev_output: Option<[BaseField; N_STATE]> = None;

    for message in messages {
        let mut state: [BaseField; N_STATE] = if let Some(prev) = prev_output {
            std::array::from_fn(|i| {
                if i < RATE {
                    prev[i] + message[i]
                } else {
                    prev[i]
                }
            })
        } else {
            std::array::from_fn(|i| {
                if i < RATE {
                    message[i]
                } else {
                    BaseField::from_u32_unchecked(0)
                }
            })
        };

        poseidon2_permutation(&mut state);
        outputs.push(state);
        prev_output = Some(state);
    }

    outputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hash() {
        let input = vec![
            BaseField::from_u32_unchecked(1),
            BaseField::from_u32_unchecked(2),
            BaseField::from_u32_unchecked(3),
        ];

        let hash = hash(&input);
        // Verify it produces a non-zero hash
        assert_ne!(hash, BaseField::from_u32_unchecked(0));
    }

    #[test]
    fn test_sponge_absorb() {
        let mut sponge = Poseidon2Sponge::new();
        sponge.absorb(BaseField::from_u32_unchecked(42));
        let hash = sponge.finalize();
        assert_ne!(hash, BaseField::from_u32_unchecked(0));
    }

    #[test]
    fn test_deterministic() {
        let input = vec![
            BaseField::from_u32_unchecked(1),
            BaseField::from_u32_unchecked(2),
            BaseField::from_u32_unchecked(3),
        ];

        let hash1 = hash(&input);
        let hash2 = hash(&input);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_inputs_different_hashes() {
        let input1 = vec![BaseField::from_u32_unchecked(1)];
        let input2 = vec![BaseField::from_u32_unchecked(2)];

        let hash1 = hash(&input1);
        let hash2 = hash(&input2);

        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_vertical_chaining() {
        let messages = vec![
            std::array::from_fn(|i| BaseField::from_u32_unchecked(i as u32)),
            std::array::from_fn(|i| BaseField::from_u32_unchecked((i + 8) as u32)),
        ];

        let outputs = hash_messages(&messages);
        assert_eq!(outputs.len(), 2);

        // Outputs should be non-zero
        assert_ne!(outputs[0][0], BaseField::from_u32_unchecked(0));
        assert_ne!(outputs[1][0], BaseField::from_u32_unchecked(0));
    }
}
