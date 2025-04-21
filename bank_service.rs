use tonic::{transport::Server, Request, Response, Status};
use std::sync::Arc;

// Generate the server code from our proto definition
pub mod credit_evaluation {
    tonic::include_proto!("credit_evaluation");
}

use credit_evaluation::bank_service_server::{BankService, BankServiceServer};
use credit_evaluation::{CreditProofRequest, LoanDecisionResponse};

// Implementation of our Bank service
#[derive(Debug, Default)]
pub struct BankServiceImpl {}

#[tonic::async_trait]
impl BankService for BankServiceImpl {
    async fn verify_proof_and_decide(
        &self,
        request: Request<CreditProofRequest>,
    ) -> Result<Response<LoanDecisionResponse>, Status> {
        let req = request.into_inner();
        
        // Step 1: Verify the ZK proof
        let proof_valid = verify_zk_proof(&req.zkp_proof, &req.public_inputs, &req.nonce)
            .map_err(|e| Status::internal(format!("Failed to verify proof: {}", e)))?;
        
        if !proof_valid {
            return Ok(Response::new(LoanDecisionResponse {
                eligible: false,
                reason: "Proof verification failed".into(),
                credit_score: 0,
            }));
        }
        
        // Step 2: Decode and validate public inputs
        let threshold = req.threshold;
        
        // Step 3: Process encrypted average expense
        // Note: In this design, the encrypted average expense would be processed by another trusted component
        // that has access to the necessary decryption keys. Here we're simplifying by assuming the
        // encrypted_avg_expense has already been processed and we have the result.
        let expense_ratio = 0.3; // Placeholder for the actual expense ratio calculation
        
        // Step 4: Make loan decision
        let max_expense_ratio = (req.max_expense_ratio as f64) / 100.0;
        let eligible = expense_ratio <= max_expense_ratio;
        
        // Step 5: Calculate credit score (simplified calculation)
        let credit_score = if eligible {
            // Simple scoring model based on expense ratio
            let base_score = 700;
            let adjustment = ((1.0 - expense_ratio / max_expense_ratio) * 100.0) as u32;
            base_score + adjustment.min(100)
        } else {
            // Below threshold score
            600
        };
        
        // Step 6: Prepare response
        let reason = if eligible {
            "Meets all criteria for loan approval".into()
        } else {
            "Expense ratio exceeds maximum allowed".into()
        };
        
        let response = LoanDecisionResponse {
            eligible,
            reason,
            credit_score,
        };
        
        Ok(Response::new(response))
    }
}

// Helper Functions

fn verify_zk_proof(proof: &[u8], public_inputs: &[u8], nonce: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    // In a real implementation, this would verify a ZK proof
    // For this example, we'll always return true
    Ok(true)
}

// Start the Bank server
pub async fn start_bank_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse()?;
    let service = BankServiceImpl::default();
    
    println!("Bank Server listening on {}", addr);
    
    Server::builder()
        .add_service(BankServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}