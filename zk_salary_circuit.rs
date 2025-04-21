use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::{
    circuit_builder::CircuitBuilder,
    circuit_data::CircuitConfig,
    config::{GenericConfig, PoseidonGoldilocksConfig},
};
use plonky2::iop::target::Target;
use plonky2::iop::target::BoolTarget;
use std::time::Instant;

// Define the configuration type
type F = <PoseidonGoldilocksConfig as GenericConfig<2>>::F;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

/// Creates a ZK circuit that proves the salary is greater than the threshold
pub fn create_salary_threshold_circuit() -> (
    CircuitBuilder<F, D>,
    Target, // salary_target
    Target, // threshold_target
    BoolTarget, // result_target (true if salary > threshold)
) {
    // Create a new circuit with default configuration
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // Create targets for inputs
    let salary_target = builder.add_virtual_target();
    let threshold_target = builder.add_virtual_target();

    // Add a range check for salary (to ensure it's a valid positive number)
    let salary_bits = builder.split_le(salary_target, 64);
    
    // Add a range check for threshold
    let threshold_bits = builder.split_le(threshold_target, 64);
    
    // Create the salary > threshold check by comparing the bits
    // Start with most significant bit (MSB) and work downwards
    let mut is_greater = builder.constant_bool(false);
    let mut is_equal_so_far = builder.constant_bool(true);
    
    // Compare bits from most significant to least significant
    for i in (0..64).rev() {
        let s_bit = salary_bits[i];
        let t_bit = threshold_bits[i];
        
        // salary_bit > threshold_bit at this position
        // equivalent to s_bit AND (NOT t_bit)
        let not_t_bit = builder.not(t_bit);
        let bit_greater = builder.and(s_bit, not_t_bit);
        
        // salary_bit == threshold_bit at this position
        let bit_equal = builder.is_equal(s_bit.target, t_bit.target);
        
        // is_greater = is_greater || (is_equal_so_far && bit_greater)
        let new_greater = builder.and(is_equal_so_far, bit_greater);
        is_greater = builder.or(is_greater, new_greater);
        
        // Update is_equal_so_far for next iteration
        is_equal_so_far = builder.and(is_equal_so_far, bit_equal);
    }
    
    // Constrain the result to be true (salary > threshold)
    builder.assert_one(is_greater.target);
    
    (builder, salary_target, threshold_target, is_greater)
}

/// Generates a ZK proof that salary > threshold
pub fn generate_salary_threshold_proof(
    salary: u64,
    threshold: u64,
) -> Result<bool, String> {
    // Only proceed if salary > threshold (otherwise we can't create a valid proof)
    if salary <= threshold {
        return Ok(false);
    }

    // Create the circuit
    let (mut builder, salary_target, threshold_target, result_target) = create_salary_threshold_circuit();
    
    // Build the circuit
    let start = Instant::now();
    let circuit_data = builder.build::<C>();
    println!("Circuit built in {:?}", start.elapsed());
    
    // Create a partial witness
    let mut pw = PartialWitness::new();
    
    // Set the private witness (salary)
    pw.set_target(salary_target, F::from_canonical_u64(salary));
    
    // Set the public input (threshold)
    pw.set_target(threshold_target, F::from_canonical_u64(threshold));
    
    // Generate the proof
    let start = Instant::now();
    let proof = circuit_data.prove(pw).map_err(|e| format!("Proving error: {:?}", e))?;
    println!("Proof generated in {:?}", start.elapsed());
    
    // Verify the proof
    let start = Instant::now();
    circuit_data
        .verify(proof)
        .map_err(|e| format!("Verification error: {:?}", e))?;
    println!("Proof verified in {:?}", start.elapsed());
    
    Ok(true)
}
