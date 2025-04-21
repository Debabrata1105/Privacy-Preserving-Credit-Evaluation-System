use std::error::Error;
use tokio;
use tonic::transport::Channel;

mod zk_salary_circuit;
mod fhe_utils;
mod nbfc_service;
mod bank_service;

use nbfc_service::credit_evaluation::nbfc_service_client::NbfcServiceClient;
use bank_service::credit_evaluation::bank_service_client::BankServiceClient;
use nbfc_service::credit_evaluation::{EncryptedFinancialRequest};
use bank_service::credit_evaluation::{CreditProofRequest};

// Main function to demonstrate the workflow
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Privacy-Preserving Credit Evaluation System");
    
    // Start servers in separate tasks
    tokio::spawn(async {
        if let Err(e) = nbfc_service::start_nbfc_server().await {
            eprintln!("NBFC server error: {}", e);
        }
    });
    
    tokio::spawn(async {
        if let Err(e) = bank_service::start_bank_server().await {
            eprintln!("Bank server error: {}", e);
        }
    });
    
    // Allow servers to start
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Run the demonstration workflow
    run_demonstration_workflow().await?;
    
    Ok(())
}

async fn run_demonstration_workflow() -> Result<(), Box<dyn Error>> {
    println!("\n=== DEMONSTRATION WORKFLOW ===\n");
    
    // Step 1: Simulate user data (in a real application, this would be collected securely)
    let salary = 6000; // Changed from 6000.0 to 6000 (u64)
    let expenses = vec![1200, 800, 350, 450, 200]; // Changed from f64 to u64
    let threshold = 5000;
    let max_expense_ratio = 50; // 50%
    
    println!("User data (for demonstration only - would be private in real system):");
    println!("  Salary: ${}", salary);
    println!("  Monthly Expenses: ${}", expenses.iter().sum::<u64>());
    println!("  Threshold: ${}", threshold);
    println!("  Max Expense Ratio: {}%", max_expense_ratio);
    
    // Step 2: Encrypt the financial data
    println!("\nEncrypting financial data...");
    let (encrypted_salary, client_key, server_key) = fhe_utils::encrypt_salary(salary)?;
    // Fix: Clone expenses to avoid ownership issues
    let encrypted_expenses = fhe_utils::encrypt_expenses(expenses.clone(), &client_key)?;
    
    // Serialize the encryption context (in a real implementation, this would be more secure)
    let encryption_context = vec![1, 2, 3, 4]; // Placeholder
    
    // Step 3: Connect to NBFC service
    println!("Connecting to NBFC service...");
    let mut nbfc_client = NbfcServiceClient::connect("http://[::1]:50051").await?;
    
    // Step 4: Send encrypted data to NBFC
    println!("Sending encrypted data to NBFC...");
    let encrypted_salary_bytes = vec![1, 2, 3, 4]; // Placeholder for serialized encrypted salary
    let encrypted_expenses_bytes: Vec<Vec<u8>> = (0..expenses.len())
        .map(|_| vec![1, 2, 3, 4]) // Placeholder for serialized encrypted expenses
        .collect();
    
    let nbfc_request = tonic::Request::new(EncryptedFinancialRequest {
        encrypted_salary: encrypted_salary_bytes,
        encrypted_expenses: encrypted_expenses_bytes,
        encryption_context,
        threshold,
    });
    
    // Step 5: Get proof from NBFC
    println!("Generating credit proof at NBFC...");
    let nbfc_response = nbfc_client.generate_credit_proof(nbfc_request).await?;
    let proof_response = nbfc_response.into_inner();
    
    println!("Proof generated successfully!");
    
    // Step 6: Connect to Bank service
    println!("\nConnecting to Bank service...");
    let mut bank_client = BankServiceClient::connect("http://[::1]:50052").await?;
    
    // Step 7: Send proof to Bank
    println!("Sending proof to Bank for verification...");
    let bank_request = tonic::Request::new(CreditProofRequest {
        zkp_proof: proof_response.zkp_proof,
        public_inputs: proof_response.public_inputs,
        encrypted_avg_expense: proof_response.encrypted_avg_expense,
        nonce: proof_response.nonce,
        threshold,
        max_expense_ratio,
    });
    
    // Step 8: Get loan decision from Bank
    println!("Getting loan decision...");
    let bank_response = bank_client.verify_proof_and_decide(bank_request).await?;
    let decision = bank_response.into_inner();
    
    // Step 9: Display results
    println!("\n=== LOAN DECISION ===");
    println!("Eligible: {}", decision.eligible);
    println!("Reason: {}", decision.reason);
    println!("Credit Score: {}", decision.credit_score);
    
    println!("\nNote: The financial data was never revealed to the Bank!");
    println!("Only proof of eligibility and differentially private expense metrics were shared.");
    
    Ok(())
}