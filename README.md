# poseidon2_m31

Pure Rust implementation of the **Poseidon2** hash function for the **M31** (Mersenne-31: 2^31 - 1) field, compatible with Stwo circuit implementation.

[![Crates.io](https://img.shields.io/crates/v/poseidon2_m31)](https://crates.io/crates/poseidon2_m31)
[![Documentation](https://docs.rs/poseidon2_m31/badge.svg)](https://docs.rs/poseidon2_m31)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

## Features

- ✅ **M31 Field**: Uses `BaseField` from the `stwo` library (2^31 - 1)
- ✅ **Poseidon2**: Based on the paper <https://eprint.iacr.org/2023/323.pdf>
- ✅ **Sponge Construction**: Automatic zero-padding
- ✅ **Vertical Chaining**: Supports message chaining (circuit-compatible)
- ✅ **100% Compatible** with Stwo AIR implementation

## Parameters

- **State size**: 16 elements (8 rate + 8 capacity)
- **Rate**: 8 elements (message absorption)
- **Capacity**: 8 elements (security)
- **Rounds**: 4 full → 14 partial → 4 full
- **Padding**: Automatic zero-padding to multiples of 8

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
poseidon2_m31 = "0.1.0"
stwo = "1.0"
```

**Note**: Requires Rust **nightly** (for `stwo` dependency):

```bash
rustup default nightly
```

## Usage

### Simple Hash (with auto-padding)

```rust
use stwo::core::fields::m31::BaseField;
use poseidon2_m31::sponge::Poseidon2Sponge;

let mut sponge = Poseidon2Sponge::new();

// Absorb any number of elements
sponge.absorb(BaseField::from_u32_unchecked(1));
sponge.absorb(BaseField::from_u32_unchecked(2));
sponge.absorb(BaseField::from_u32_unchecked(3));

// Auto-pads to 8 elements: [1, 2, 3, 0, 0, 0, 0, 0]
let hash = sponge.finalize();
println!("Hash: {}", hash.0);
```

### Hash Full Message (8 elements)

```rust
use poseidon2_m31::sponge::hash_messages;
use poseidon2_m31::poseidon2::RATE;

let message: [BaseField; RATE] = std::array::from_fn(|i| {
    BaseField::from_u32_unchecked(i as u32)
});

let outputs = hash_messages(&[message]);
println!("Hash: {}", outputs[0][0].0);
```

### Vertical Chaining (multiple messages)

```rust
let messages = vec![
    std::array::from_fn(|i| BaseField::from_u32_unchecked(i as u32)),
    std::array::from_fn(|i| BaseField::from_u32_unchecked((i + 8) as u32)),
];

let outputs = hash_messages(&messages);
// Each message produces a full state (16 elements)
// output[1] depends on output[0] (chaining)
```

### Full State Output

```rust
let full_state = sponge.finalize_full_state();
// Returns all 16 state elements
```

## Examples

Run the example:

```bash
cargo run --example simple_hash --release
```

Or run the main binary:

```bash
cargo run --release
```

## Testing

```bash
# All tests (12 compatibility tests + 8 unit tests)
cargo test --release

# Tests with output
cargo test --release -- --nocapture
```

## API Reference

### `Poseidon2Sponge::new() -> Self`
Creates a new hasher with zero state.

### `absorb(&mut self, element: BaseField)`
Absorbs a single element. Automatically processes blocks of 8 elements.

### `absorb_many(&mut self, elements: &[BaseField])`
Absorbs multiple elements.

### `finalize(self) -> BaseField`
Finalizes and returns the hash (first state element). Automatically pads with zeros if needed.

### `finalize_full_state(self) -> [BaseField; 16]`
Returns the full state (16 elements).

### `hash_messages(messages: &[[BaseField; 8]]) -> Vec<[BaseField; 16]>`
Hashes a sequence of messages with vertical chaining (identical to circuit).

## Compatibility with Stwo Circuit

This implementation is a **1:1 copy** of the circuit logic:
- ✅ Same constants (`EXTERNAL_ROUND_CONSTS`, `INTERNAL_ROUND_CONSTS`)
- ✅ Same `BaseField` type (M31)
- ✅ Same operation order
- ✅ Same vertical chaining
- ✅ **Identical hash results**

## Padding

**Automatic zero-padding:**
- Input `[1, 2, 3]` → hash of `[1, 2, 3, 0, 0, 0, 0, 0]`
- Input `[1..8]` → hash of `[1..8]` (full block, no padding)
- Input `[1..10]` → hash of `[1..8]` + `[9, 10, 0, 0, 0, 0, 0, 0]` (2 blocks)

## Verified Compatibility

Tested and verified against circuit for:
- ✅ Single message [0..7]
- ✅ Two messages with chaining
- ✅ Three messages with chaining
- ✅ Padding (3 elements → 8 with zeros)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or pull request.

## References

- Poseidon2 Paper: <https://eprint.iacr.org/2023/323.pdf>
- Stwo: <https://github.com/starkware-libs/stwo>
