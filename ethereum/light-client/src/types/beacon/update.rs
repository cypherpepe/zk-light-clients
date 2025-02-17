// Copyright (c) Argument Computer Corporation
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use ethereum_lc_core::types::error::TypesError;
use ethereum_lc_core::types::update::{Update, UPDATE_BASE_BYTES_LEN};
use ethereum_lc_core::types::utils::{calc_sync_period, U64_LEN};
use ethereum_lc_core::types::ForkDigest;
use getset::Getters;

/// Payload received from the Beacon Node when fetching updates starting at a given period.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct UpdateResponse {
    pub updates: Vec<UpdateItem>,
}

impl UpdateResponse {
    /// Deserialize a `UpdateResponse` from SSZ bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The SSZ encoded bytes.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized `UpdateResponse` or a `TypesError`.
    pub fn from_ssz_bytes(bytes: &[u8]) -> Result<UpdateResponse, TypesError> {
        let mut cursor = 0;
        let mut updates = vec![];

        while cursor < bytes.len() {
            if cursor + U64_LEN >= bytes.len() {
                return Err(TypesError::UnderLength {
                    minimum: U64_LEN + 4 + UPDATE_BASE_BYTES_LEN,
                    actual: bytes.len(),
                    structure: "UpdateResponse".into(),
                });
            }

            let size = u64::from_le_bytes(bytes[cursor..cursor + U64_LEN].try_into().unwrap());
            cursor += U64_LEN;

            if cursor + size as usize > bytes.len() {
                return Err(TypesError::UnderLength {
                    minimum: cursor + size as usize,
                    actual: bytes.len(),
                    structure: "UpdateResponse".into(),
                });
            }

            let fork_digest: [u8; 4] = bytes[cursor..cursor + 4].try_into().unwrap();

            let update = Update::from_ssz_bytes(&bytes[cursor + 4..cursor + size as usize])?;
            cursor += size as usize;

            updates.push(UpdateItem {
                size,
                fork_digest,
                update,
            });
        }

        Ok(UpdateResponse { updates })
    }

    /// Returns the update containing the committee change for the given period.
    ///
    /// # Arguments
    ///
    /// * `known_period` - The period for which we already have the committee.
    ///
    /// # Returns
    ///
    /// An `Option` containing the update if it exists.
    pub fn extract_committee_change(self, known_period: u64) -> Result<Option<Update>> {
        for update_item in self.updates {
            let update_period =
                calc_sync_period(update_item.update.attested_header().beacon().slot());
            if update_period == known_period + 1 {
                return Ok(Some(update_item.update));
            }
        }
        Ok(None)
    }
}

/// An item in the `UpdateResponse` containing the size of the update, the fork digest and the update itself.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct UpdateItem {
    pub size: u64,
    pub fork_digest: ForkDigest,
    pub update: Update,
}
