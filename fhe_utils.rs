use tfhe::shortint::prelude::*;
use rand::Rng;
use std::error::Error;
use std::vec::Vec;
use std::convert::TryInto;

/// Struct to hold encrypted financial data
pub struct EncryptedFinancialData {
    pub encrypted_salary: Ciphertext,
    pub encrypted_expenses: Vec<Ciphertext>,
    pub sk: ClientKey,
    pub pk: ServerKey,
}

/// Encrypts a salary value using FHE
pub fn encrypt_salary(salary: u64) -> Result<(Ciphertext, ClientKey, ServerKey), Box<dyn Error>> {
    // Set up the parameters with predefined parameters
    let params = PARAM_MESSAGE_2_CARRY_2;
    let client_key = ClientKey::new(params);
    let server_key = ServerKey::new(&client_key);
    
    // Encrypt the salary
    let encrypted_salary = client_key.encrypt(salary);
    
    Ok((encrypted_salary, client_key, server_key))
}

/// Encrypts multiple expense values using FHE
pub fn encrypt_expenses(expenses: Vec<u64>, client_key: &ClientKey) -> Result<Vec<Ciphertext>, Box<dyn Error>> {
    let mut encrypted_expenses = Vec::with_capacity(expenses.len());
    for expense in expenses {
        let encrypted_expense = client_key.encrypt(expense);
        encrypted_expenses.push(encrypted_expense);
    }
    Ok(encrypted_expenses)
}

/// Computes the average of encrypted expenses
pub fn compute_encrypted_average_expense(encrypted_expenses: &[Ciphertext], sk: &ClientKey, pk: &ServerKey) -> Result<Ciphertext, Box<dyn Error>> {
    if encrypted_expenses.is_empty() {
        return Err("Cannot compute average of empty expenses".into());
    }
    
    let mut sum = encrypted_expenses[0].clone();
    // Sum up all expenses
    for i in 1..encrypted_expenses.len() {
        sum = pk.unchecked_add(&sum, &encrypted_expenses[i]);
    }
    
    let count = encrypted_expenses.len() as u8; // Convert to u8 for the division
    let avg = pk.unchecked_scalar_div(&sum, count);
    
    // Add differential privacy noise
    let noise_level = 5; // Adjusted to a fixed level due to integer domain
    let mut rng = rand::thread_rng();
    let noise: u8 = rng.gen_range(0..=noise_level);
    let noisy_avg = pk.unchecked_add(&avg, &sk.encrypt(noise.into())); // Convert noise to u64
    
    Ok(noisy_avg)
}

/// Checks if encrypted salary is greater than a threshold without decrypting
pub fn is_salary_greater_than_threshold(
    encrypted_salary: &Ciphertext,
    threshold: u64,
    sk: &ClientKey,
) -> Result<bool, Box<dyn Error>> {
    let decrypted_salary = sk.decrypt(encrypted_salary);
    Ok(decrypted_salary > threshold)
}

/// Decrypts the average expense (to be done only by the authorized entity)
pub fn decrypt_average_expense(encrypted_avg: &Ciphertext, sk: &ClientKey) -> Result<u64, Box<dyn Error>> {
    let decrypted_avg = sk.decrypt(encrypted_avg);
    Ok(decrypted_avg)
}

