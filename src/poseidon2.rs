//! Poseidon2 hash function implementation for M31 field.
//!
//! Based on the paper: <https://eprint.iacr.org/2023/323.pdf>

use std::ops::{Add, AddAssign, Mul, Sub};
use stwo::core::fields::m31::BaseField;

pub const N_STATE: usize = 16;
pub const RATE: usize = 8;
pub const CAPACITY: usize = 8;
pub const N_PARTIAL_ROUNDS: usize = 14;
pub const N_HALF_FULL_ROUNDS: usize = 4;
pub const FULL_ROUNDS: usize = 2 * N_HALF_FULL_ROUNDS;

pub const EXTERNAL_ROUND_CONSTS: [[BaseField; N_STATE]; FULL_ROUNDS] =
    [[BaseField::from_u32_unchecked(1234); N_STATE]; FULL_ROUNDS];

pub const INTERNAL_ROUND_CONSTS: [BaseField; N_PARTIAL_ROUNDS] =
    [BaseField::from_u32_unchecked(1234); N_PARTIAL_ROUNDS];

/// Applies x^5 S-box
#[inline]
pub fn pow5<F>(x: F) -> F
where
    F: Clone + Mul<Output = F>,
{
    let x2 = x.clone() * x.clone();
    let x4 = x2.clone() * x2.clone();
    x4 * x
}

/// Applies the M4 MDS matrix (section 5.1)
#[inline]
pub fn apply_m4<F>(x: [F; 4]) -> [F; 4]
where
    F: Clone + AddAssign<F> + Add<F, Output = F> + Sub<F, Output = F> + Mul<BaseField, Output = F>,
{
    let t0 = x[0].clone() + x[1].clone();
    let t02 = t0.clone() + t0.clone();
    let t1 = x[2].clone() + x[3].clone();
    let t12 = t1.clone() + t1.clone();
    let t2 = x[1].clone() + x[1].clone() + t1.clone();
    let t3 = x[3].clone() + x[3].clone() + t0.clone();
    let t4 = t12.clone() + t12.clone() + t3.clone();
    let t5 = t02.clone() + t02.clone() + t2.clone();
    let t6 = t3.clone() + t5.clone();
    let t7 = t2.clone() + t4.clone();
    [t6, t5, t7, t4]
}

/// Applies the external round matrix (section 5.1, Appendix B)
pub fn apply_external_round_matrix<F>(state: &mut [F; N_STATE])
where
    F: Clone + AddAssign<F> + Add<F, Output = F> + Sub<F, Output = F> + Mul<BaseField, Output = F>,
{
    // Apply circ(2M4, M4, M4, M4)
    for i in 0..4 {
        [
            state[4 * i],
            state[4 * i + 1],
            state[4 * i + 2],
            state[4 * i + 3],
        ] = apply_m4([
            state[4 * i].clone(),
            state[4 * i + 1].clone(),
            state[4 * i + 2].clone(),
            state[4 * i + 3].clone(),
        ]);
    }

    // Apply column mixing
    for j in 0..4 {
        let s =
            state[j].clone() + state[j + 4].clone() + state[j + 8].clone() + state[j + 12].clone();
        for i in 0..4 {
            state[4 * i + j] += s.clone();
        }
    }
}

/// Applies the internal round matrix (section 5.2)
pub fn apply_internal_round_matrix<F>(state: &mut [F; N_STATE])
where
    F: Clone + AddAssign<F> + Add<F, Output = F> + Sub<F, Output = F> + Mul<BaseField, Output = F>,
{
    let sum = state[1..]
        .iter()
        .cloned()
        .fold(state[0].clone(), |acc, s| acc + s);

    state.iter_mut().enumerate().for_each(|(i, s)| {
        *s = s.clone() * BaseField::from_u32_unchecked(1 << (i + 1)) + sum.clone();
    });
}

/// Applies the Poseidon2 permutation to a state
pub fn poseidon2_permutation(state: &mut [BaseField; N_STATE]) {
    // First 4 full rounds
    for round in 0..N_HALF_FULL_ROUNDS {
        // Add round constants
        for i in 0..N_STATE {
            state[i] += EXTERNAL_ROUND_CONSTS[round][i];
        }
        // Apply external matrix
        apply_external_round_matrix(state);
        // Apply S-box (x^5) to all elements
        *state = std::array::from_fn(|i| pow5(state[i]));
    }

    // Partial rounds (only first element gets S-box)
    for round in 0..N_PARTIAL_ROUNDS {
        state[0] += INTERNAL_ROUND_CONSTS[round];
        apply_internal_round_matrix(state);
        state[0] = pow5(state[0]);
    }

    // Last 4 full rounds
    for round in 0..N_HALF_FULL_ROUNDS {
        // Add round constants
        for i in 0..N_STATE {
            state[i] += EXTERNAL_ROUND_CONSTS[round + N_HALF_FULL_ROUNDS][i];
        }
        // Apply external matrix
        apply_external_round_matrix(state);
        // Apply S-box (x^5) to all elements
        *state = std::array::from_fn(|i| pow5(state[i]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow5() {
        let x = BaseField::from_u32_unchecked(3);
        let result = pow5(x);
        // 3^5 = 243
        assert_eq!(result, BaseField::from_u32_unchecked(243));
    }

    #[test]
    fn test_m4_matrix() {
        let input = [
            BaseField::from_u32_unchecked(1),
            BaseField::from_u32_unchecked(2),
            BaseField::from_u32_unchecked(3),
            BaseField::from_u32_unchecked(4),
        ];
        let output = apply_m4(input);
        // Just verify it runs without panicking
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_permutation() {
        let mut state = std::array::from_fn(|i| BaseField::from_u32_unchecked(i as u32));
        poseidon2_permutation(&mut state);
        // Verify state changed (permutation is not identity)
        assert_ne!(state[0], BaseField::from_u32_unchecked(0));
    }
}
