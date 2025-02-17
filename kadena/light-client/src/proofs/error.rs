// Copyright (c) Argument Computer Corporation
// SPDX-License-Identifier: APACHE-2.0

use thiserror::Error;

/// The error type for errors being thrown while proving program execution.
#[derive(Debug, Error)]
pub enum ProverError {
    #[error("Error while generating Sphinx input: {source}")]
    SphinxInput {
        #[source]
        source: Box<dyn std::error::Error + Sync + Send>,
    },
    #[error("Error while executing the program: {source}")]
    Execution {
        #[source]
        source: Box<dyn std::error::Error + Sync + Send>,
    },
    #[error("Error while generating {proof_type} the program: {source}")]
    Proving {
        proof_type: String,
        #[source]
        source: Box<dyn std::error::Error + Sync + Send>,
    },
    #[error("Error while verifying the proof: {source}")]
    Verification {
        #[source]
        source: Box<dyn std::error::Error + Sync + Send>,
    },
}

/// The error type for the errors being thrown when reading public values from a proof.
#[derive(Debug, Error)]
pub enum PublicValuesError {
    #[error("Error while reading public values from Sphinx proof: {source}")]
    SphinxProof {
        #[source]
        source: Box<dyn std::error::Error + Sync + Send>,
    },
}
