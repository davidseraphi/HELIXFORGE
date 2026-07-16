//! Classification and crypto policy for sovereign Collab.

use helix_db::{forbids_cleartext, requires_client_e2ee, validate_classification};
use shared_core::{HelixError, HelixResult};

/// Enforce that a document mutation is allowed under classification + crypto flags.
pub fn enforce_write_crypto(
    classification: &str,
    client_e2ee: bool,
    server_vault_e2ee: bool,
    content_is_client_envelope: bool,
) -> HelixResult<()> {
    validate_classification(classification)?;
    if requires_client_e2ee(classification) && !client_e2ee {
        return Err(HelixError::forbidden(
            "classification requires client_e2ee (server vault forbidden)",
        ));
    }
    if requires_client_e2ee(classification) && server_vault_e2ee && !client_e2ee {
        return Err(HelixError::forbidden(
            "server vault e2ee not allowed for restricted/sovereign",
        ));
    }
    if forbids_cleartext(classification) && !client_e2ee && !server_vault_e2ee {
        return Err(HelixError::forbidden(
            "classification forbids cleartext durable storage",
        ));
    }
    if client_e2ee && !content_is_client_envelope {
        return Err(HelixError::validation(
            "client_e2ee content must be HC1 envelope",
        ));
    }
    Ok(())
}

#[allow(dead_code)]
pub fn parse_class(s: &str) -> HelixResult<&str> {
    validate_classification(s)?;
    Ok(s)
}
