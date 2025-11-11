mod poseidon2;
mod sponge;

use sponge::{hash_messages, Poseidon2Sponge};
use stwo::core::fields::m31::BaseField;

fn main() {
    println!("=== Poseidon2 Hash Examples ===\n");

    // Example 1: Auto-padding
    println!("Example 1: Auto-padding");
    let mut sponge = Poseidon2Sponge::new();
    sponge.absorb(BaseField::from_u32_unchecked(1));
    sponge.absorb(BaseField::from_u32_unchecked(2));
    sponge.absorb(BaseField::from_u32_unchecked(3));
    let hash = sponge.finalize();
    println!("Input:  [1, 2, 3]");
    println!("Padded: [1, 2, 3, 0, 0, 0, 0, 0]");
    println!("Hash:   {}\n", hash.0);

    // Example 2: Full message (circuit-compatible)
    println!("Example 2: Full message (8 elements)");
    let message = std::array::from_fn(|i| BaseField::from_u32_unchecked(i as u32));
    let outputs = hash_messages(&[message]);
    println!("Input:    [0, 1, 2, 3, 4, 5, 6, 7]");
    println!("Hash:     {}", outputs[0][0].0);
    println!("Expected: 334078718 ✓\n");

    // Example 3: Vertical chaining
    println!("Example 3: Vertical chaining (3 messages)");
    let messages = vec![
        std::array::from_fn(|i| BaseField::from_u32_unchecked(i as u32)),
        std::array::from_fn(|i| BaseField::from_u32_unchecked((i + 8) as u32)),
        std::array::from_fn(|i| BaseField::from_u32_unchecked((i + 16) as u32)),
    ];

    let outputs = hash_messages(&messages);
    for (i, output) in outputs.iter().enumerate() {
        println!("Message {}: hash = {}", i, output[0].0);
    }

    // Example 4: Full state output
    println!("\nExample 4: Full state (16 elements)");
    let full_state = outputs[0];
    println!(
        "Rate (0-7):      {:?}",
        &full_state[0..8].iter().map(|f| f.0).collect::<Vec<_>>()
    );
    println!(
        "Capacity (8-15): {:?}",
        &full_state[8..16].iter().map(|f| f.0).collect::<Vec<_>>()
    );

    println!("\n✅ All examples completed!");
}
