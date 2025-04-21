use tonic::{transport::Server, Request, Response, Status};
use bincode;
use std::sync::Arc;

// Generate the server code from our proto definition
pub mod credit_evaluation {
    tonic::include_proto!("credit_evaluation");
}

use credit_evaluation::nbfc_service_server::{NbfcService, NbfcServiceServer};
use credit_evaluation::{EncryptedFinancialRequest, CreditProofResponse};

// Import our custom modules
use crate::fhe_utils;
use crate::zk_salary_circuit::generate_salary_threshold_proof;

// Implementation of our NBFC service
#[derive(Debug, Default)]
pub struct NBFCServiceImpl {}

#[tonic::async_trait]
impl NbfcService for NBFCServiceImpl {
    async fn generate_credit_proof(
        &self,
        request: Request<EncryptedFinancialRequest>,
    ) -> Result<Response<CreditProofResponse>, Status> {
        let req = request.into_inner();
        
        // Step 1: Deserialize encryption context
        let encryption_context = deserialize_encryption_context(&req.encryption_context)
            .map_err(|e| Status::internal(format!("Failed to deserialize encryption context: {}", e)))?;
        
        // Step 2: Extract encrypted data
        let encrypted_salary = deserialize_lwe(&req.encrypted_salary)
            .map_err(|e| Status::internal(format!("Failed to deserialize encrypted salary: {}", e)))?;
        
        let encrypted_expenses: Vec<_> = req.encrypted_expenses
            .iter()
            .map(|e| deserialize_lwe(e))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::internal(format!("Failed to deserialize encrypted expenses: {}", e)))?;
        
        // Step 3: Decrypt salary internally (only within NBFC, never shared)
        // In a real implementation, this would be done using secure enclaves or MPC
        let salary = decrypt_salary(&encrypted_salary, &encryption_context)
            .map_err(|e| Status::internal(format!("Failed to process salary: {}", e)))?;
        
        // Step 4: Compute average expense (remains encrypted)
        let encrypted_avg_expense = compute_encrypted_average_expense(
            &encrypted_expenses, 
            &encryption_context.encoder, 
            &encryption_context.sk
        ).map_err(|e| Status::internal(format!("Failed to compute average expense: {}", e)))?;
        
        // Step 5: Generate ZK proof that salary > threshold
        let threshold = req.threshold;
        let proof_successful = generate_salary_threshold_proof(salary as u64, threshold)
            .map_err(|e| Status::internal(format!("Failed to generate proof: {}", e)))?;
        
        if !proof_successful {
            return Err(Status::invalid_argument("Salary does not meet threshold requirements"));
        }
        
        // Step 6: Prepare the response
        // In a real implementation, this would include properly serialized ZK proof
        let zkp_proof = vec![1, 2, 3, 4]; // Placeholder for actual proof
        let nonce = generate_nonce();
        let public_inputs = serialize_public_inputs(threshold);
        
        let response = CreditProofResponse {
            zkp_proof,
            encrypted_avg_expense: serialize_lwe(&encrypted_avg_expense),
            nonce,
            public_inputs,
        };
        
        Ok(Response::new(response))
    }
}

// Helper Functions

fn deserialize_encryption_context(data: &[u8]) -> Result<EncryptionContext, Box<dyn std::error::Error>> {
    Ok(EncryptionContext::default())
}

fn deserialize_lwe(data: &[u8]) -> Result<LWE, Box<dyn std::error::Error>> {
    Ok(LWE::default())
}

fn decrypt_salary(encrypted_salary: &LWE, context: &EncryptionContext) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(6000.0)
}

fn compute_encrypted_average_expense(
    encrypted_expenses: &[LWE],
    encoder: &Encoder,
    sk: &LWESecretKey,
) -> Result<LWE, Box<dyn std::error::Error>> {
    Ok(LWE::default())
}

fn serialize_lwe(lwe: &LWE) -> Vec<u8> {
    vec![1, 2, 3, 4]
}

fn generate_nonce() -> Vec<u8> {
    // Generate a random nonce for proof verification
    vec![5, 6, 7, 8]
}

fn serialize_public_inputs(threshold: u64) -> Vec<u8> {
    // Serialize the public inputs for the ZK proof
    let mut buf = Vec::new();
    buf.extend_from_slice(&threshold.to_le_bytes());
    buf
}

// Placeholder structs for demonstration
#[derive(Debug, Default)]
struct EncryptionContext {
    encoder: Encoder,
    sk: LWESecretKey,
}

#[derive(Debug, Default, Clone)]
struct LWE;

#[derive(Debug, Default)]
struct Encoder;

#[derive(Debug, Default)]
struct LWESecretKey;

// Start the NBFC server
pub async fn start_nbfc_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = NBFCServiceImpl::default();
    
    println!("NBFC Server listening on {}", addr);
    
    Server::builder()
        .add_service(NbfcServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}