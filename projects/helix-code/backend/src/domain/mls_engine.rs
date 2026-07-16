//! OpenMLS (RFC 9420) engine for HelixCode forge OpenMLS (RFC 9420).

use base64::Engine;
use openmls::prelude::tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::types::Ciphersuite;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use shared_core::{HelixError, HelixResult};
use std::collections::HashMap;
use std::sync::Arc;

const CS: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

type Provider = OpenMlsRustCrypto;

struct UserMls {
    provider: Arc<Provider>,
    signer: SignatureKeyPair,
    credential: CredentialWithKey,
}

#[derive(Clone, Default)]
pub struct MlsEngine {
    users: Arc<Mutex<HashMap<String, UserMls>>>,
}

impl MlsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_identity(&self, user_key: &str, label: &str) -> HelixResult<MlsIdentityOut> {
        let mut g = self.users.lock();
        if let Some(u) = g.get(user_key) {
            let pub_b64 =
                base64::engine::general_purpose::STANDARD.encode(u.signer.to_public_vec());
            return Ok(MlsIdentityOut {
                user_key: user_key.into(),
                label: label.into(),
                signature_public_b64: pub_b64,
            });
        }
        let provider = Arc::new(Provider::default());
        let identity = format!("{user_key}:{label}").into_bytes();
        let credential = BasicCredential::new(identity);
        let signature_keys = SignatureKeyPair::new(CS.signature_algorithm())
            .map_err(|e| HelixError::internal(format!("mls sig keys: {e:?}")))?;
        signature_keys
            .store(provider.storage())
            .map_err(|e| HelixError::internal(format!("mls store sig: {e:?}")))?;
        let credential_with_key = CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keys.to_public_vec().into(),
        };
        let pub_b64 =
            base64::engine::general_purpose::STANDARD.encode(signature_keys.to_public_vec());
        g.insert(
            user_key.to_string(),
            UserMls {
                provider,
                signer: signature_keys,
                credential: credential_with_key,
            },
        );
        Ok(MlsIdentityOut {
            user_key: user_key.into(),
            label: label.into(),
            signature_public_b64: pub_b64,
        })
    }

    pub fn create_key_package(&self, user_key: &str) -> HelixResult<Vec<u8>> {
        let g = self.users.lock();
        let u = g
            .get(user_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let kp = KeyPackage::builder()
            .build(CS, u.provider.as_ref(), &u.signer, u.credential.clone())
            .map_err(|e| HelixError::internal(format!("key package: {e:?}")))?;
        let mut buf = Vec::new();
        kp.key_package()
            .tls_serialize(&mut buf)
            .map_err(|e| HelixError::internal(format!("kp serialize: {e}")))?;
        Ok(buf)
    }

    pub fn create_group(&self, owner_key: &str, group_id: &str) -> HelixResult<MlsGroupInfoOut> {
        let g = self.users.lock();
        let u = g
            .get(owner_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let gid = GroupId::from_slice(group_id.as_bytes());
        let config = MlsGroupCreateConfig::builder()
            .ciphersuite(CS)
            .use_ratchet_tree_extension(true)
            .build();
        let group = MlsGroup::new_with_group_id(
            u.provider.as_ref(),
            &u.signer,
            &config,
            gid,
            u.credential.clone(),
        )
        .map_err(|e| HelixError::internal(format!("mls new group: {e:?}")))?;
        let epoch = group.epoch().as_u64();
        let members: Vec<String> = group
            .members()
            .map(|m| String::from_utf8_lossy(m.credential.serialized_content()).into_owned())
            .collect();
        let secret = group
            .export_secret(u.provider.crypto(), "helix-code-dek", &[], 32)
            .map_err(|e| HelixError::internal(format!("export secret: {e:?}")))?;
        Ok(MlsGroupInfoOut {
            group_id: group_id.into(),
            epoch,
            members,
            exported_secret_b64: base64::engine::general_purpose::STANDARD.encode(secret),
            member_count: 1,
        })
    }

    pub fn add_member(
        &self,
        owner_key: &str,
        group_id: &str,
        key_package_tls: &[u8],
    ) -> HelixResult<MlsAddOut> {
        let g = self.users.lock();
        let u = g
            .get(owner_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let gid = GroupId::from_slice(group_id.as_bytes());
        let mut group = MlsGroup::load(u.provider.storage(), &gid)
            .map_err(|e| HelixError::internal(format!("load group: {e:?}")))?
            .ok_or_else(|| HelixError::not_found("mls group"))?;
        let mut cursor = std::io::Cursor::new(key_package_tls);
        let kp = KeyPackageIn::tls_deserialize(&mut cursor)
            .map_err(|e| HelixError::validation(format!("bad key package: {e}")))?;
        let kp = kp
            .validate(u.provider.crypto(), ProtocolVersion::Mls10)
            .map_err(|e| HelixError::validation(format!("kp validate: {e:?}")))?;
        let (commit, welcome, _gi) = group
            .add_members(u.provider.as_ref(), &u.signer, &[kp])
            .map_err(|e| HelixError::internal(format!("add members: {e:?}")))?;
        group
            .merge_pending_commit(u.provider.as_ref())
            .map_err(|e| HelixError::internal(format!("merge: {e:?}")))?;
        let commit_bytes = commit
            .to_bytes()
            .map_err(|e| HelixError::internal(format!("commit bytes: {e:?}")))?;
        let welcome_msg: MlsMessageOut = welcome;
        let welcome_bytes = welcome_msg
            .to_bytes()
            .map_err(|e| HelixError::internal(format!("welcome bytes: {e:?}")))?;
        let epoch = group.epoch().as_u64();
        let secret = group
            .export_secret(u.provider.crypto(), "helix-code-dek", &[], 32)
            .map_err(|e| HelixError::internal(format!("export secret: {e:?}")))?;
        let members: Vec<String> = group
            .members()
            .map(|m| String::from_utf8_lossy(m.credential.serialized_content()).into_owned())
            .collect();
        Ok(MlsAddOut {
            commit_tls_b64: base64::engine::general_purpose::STANDARD.encode(&commit_bytes),
            welcome_tls_b64: base64::engine::general_purpose::STANDARD.encode(&welcome_bytes),
            epoch,
            members,
            exported_secret_b64: base64::engine::general_purpose::STANDARD.encode(secret),
        })
    }

    pub fn join_with_welcome(
        &self,
        joiner_key: &str,
        welcome_tls: &[u8],
        ratchet_tree: Option<RatchetTreeIn>,
    ) -> HelixResult<MlsGroupInfoOut> {
        let g = self.users.lock();
        let u = g
            .get(joiner_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let mut cursor = std::io::Cursor::new(welcome_tls);
        let msg = MlsMessageIn::tls_deserialize(&mut cursor)
            .map_err(|e| HelixError::validation(format!("welcome decode: {e}")))?;
        let welcome = match msg.extract() {
            MlsMessageBodyIn::Welcome(w) => w,
            _ => return Err(HelixError::validation("expected welcome message")),
        };
        let join_cfg = MlsGroupJoinConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();
        let staged =
            StagedWelcome::new_from_welcome(u.provider.as_ref(), &join_cfg, welcome, ratchet_tree)
                .map_err(|e| HelixError::internal(format!("staged welcome: {e:?}")))?;
        let group = staged
            .into_group(u.provider.as_ref())
            .map_err(|e| HelixError::internal(format!("join group: {e:?}")))?;
        let group_id = String::from_utf8_lossy(group.group_id().as_slice()).into_owned();
        let epoch = group.epoch().as_u64();
        let members: Vec<String> = group
            .members()
            .map(|m| String::from_utf8_lossy(m.credential.serialized_content()).into_owned())
            .collect();
        let secret = group
            .export_secret(u.provider.crypto(), "helix-code-dek", &[], 32)
            .map_err(|e| HelixError::internal(format!("export secret: {e:?}")))?;
        Ok(MlsGroupInfoOut {
            group_id,
            epoch,
            members: members.clone(),
            exported_secret_b64: base64::engine::general_purpose::STANDARD.encode(secret),
            member_count: members.len(),
        })
    }

    pub fn create_app_message(
        &self,
        user_key: &str,
        group_id: &str,
        plaintext: &[u8],
    ) -> HelixResult<Vec<u8>> {
        let g = self.users.lock();
        let u = g
            .get(user_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let gid = GroupId::from_slice(group_id.as_bytes());
        let mut group = MlsGroup::load(u.provider.storage(), &gid)
            .map_err(|e| HelixError::internal(format!("load group: {e:?}")))?
            .ok_or_else(|| HelixError::not_found("mls group"))?;
        let out = group
            .create_message(u.provider.as_ref(), &u.signer, plaintext)
            .map_err(|e| HelixError::internal(format!("create msg: {e:?}")))?;
        out.to_bytes()
            .map_err(|e| HelixError::internal(format!("msg bytes: {e:?}")))
    }

    pub fn process_app_message(
        &self,
        user_key: &str,
        group_id: &str,
        message_tls: &[u8],
    ) -> HelixResult<Option<Vec<u8>>> {
        let g = self.users.lock();
        let u = g
            .get(user_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let gid = GroupId::from_slice(group_id.as_bytes());
        let mut group = MlsGroup::load(u.provider.storage(), &gid)
            .map_err(|e| HelixError::internal(format!("load group: {e:?}")))?
            .ok_or_else(|| HelixError::not_found("mls group"))?;
        let mut cursor = std::io::Cursor::new(message_tls);
        let msg = MlsMessageIn::tls_deserialize(&mut cursor)
            .map_err(|e| HelixError::validation(format!("msg decode: {e}")))?;
        let protocol = msg
            .try_into_protocol_message()
            .map_err(|e| HelixError::validation(format!("not protocol msg: {e:?}")))?;
        let processed = group
            .process_message(u.provider.as_ref(), protocol)
            .map_err(|e| HelixError::internal(format!("process: {e:?}")))?;
        match processed.into_content() {
            ProcessedMessageContent::ApplicationMessage(app) => Ok(Some(app.into_bytes())),
            ProcessedMessageContent::StagedCommitMessage(staged) => {
                group
                    .merge_staged_commit(u.provider.as_ref(), *staged)
                    .map_err(|e| HelixError::internal(format!("merge staged: {e:?}")))?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn export_group_secret(&self, user_key: &str, group_id: &str) -> HelixResult<Vec<u8>> {
        let g = self.users.lock();
        let u = g
            .get(user_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let gid = GroupId::from_slice(group_id.as_bytes());
        let group = MlsGroup::load(u.provider.storage(), &gid)
            .map_err(|e| HelixError::internal(format!("load group: {e:?}")))?
            .ok_or_else(|| HelixError::not_found("mls group"))?;
        group
            .export_secret(u.provider.crypto(), "helix-code-dek", &[], 32)
            .map_err(|e| HelixError::internal(format!("export secret: {e:?}")))
    }

    pub fn group_info(&self, user_key: &str, group_id: &str) -> HelixResult<MlsGroupInfoOut> {
        let g = self.users.lock();
        let u = g
            .get(user_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let gid = GroupId::from_slice(group_id.as_bytes());
        let group = MlsGroup::load(u.provider.storage(), &gid)
            .map_err(|e| HelixError::internal(format!("load group: {e:?}")))?
            .ok_or_else(|| HelixError::not_found("mls group"))?;
        let members: Vec<String> = group
            .members()
            .map(|m| String::from_utf8_lossy(m.credential.serialized_content()).into_owned())
            .collect();
        let secret = group
            .export_secret(u.provider.crypto(), "helix-code-dek", &[], 32)
            .map_err(|e| HelixError::internal(format!("export secret: {e:?}")))?;
        Ok(MlsGroupInfoOut {
            group_id: group_id.into(),
            epoch: group.epoch().as_u64(),
            member_count: members.len(),
            members,
            exported_secret_b64: base64::engine::general_purpose::STANDARD.encode(secret),
        })
    }

    /// Snapshot full user OpenMLS state (signer + memory keystore) for durable Postgres.
    pub fn export_user_blob(&self, user_key: &str) -> HelixResult<Vec<u8>> {
        let g = self.users.lock();
        let u = g
            .get(user_key)
            .ok_or_else(|| HelixError::validation("mls identity required"))?;
        let storage = u.provider.storage();
        let map = storage
            .values
            .read()
            .map_err(|_| HelixError::internal("mls storage lock poisoned"))?;
        let mut values = std::collections::HashMap::new();
        for (k, v) in map.iter() {
            values.insert(
                base64::engine::general_purpose::STANDARD.encode(k),
                base64::engine::general_purpose::STANDARD.encode(v),
            );
        }
        let identity = u.credential.credential.serialized_content().to_vec();
        let signer_json = serde_json::to_vec(&u.signer)
            .map_err(|e| HelixError::internal(format!("signer ser: {e}")))?;
        let dump = UserDump {
            version: 1,
            label: String::from_utf8_lossy(&identity).into_owned(),
            signer_json,
            identity,
            storage: values,
        };
        serde_json::to_vec(&dump).map_err(|e| HelixError::internal(format!("mls dump: {e}")))
    }

    /// Restore user OpenMLS state from durable blob (idempotent overwrite).
    pub fn import_user_blob(&self, user_key: &str, blob: &[u8]) -> HelixResult<()> {
        let dump: UserDump = serde_json::from_slice(blob)
            .map_err(|e| HelixError::validation(format!("mls blob: {e}")))?;
        let provider = Arc::new(Provider::default());
        {
            let storage = provider.storage();
            let mut map = storage
                .values
                .write()
                .map_err(|_| HelixError::internal("mls storage lock poisoned"))?;
            map.clear();
            for (k, v) in dump.storage {
                let kb = base64::engine::general_purpose::STANDARD
                    .decode(k)
                    .map_err(|e| HelixError::validation(format!("storage key b64: {e}")))?;
                let vb = base64::engine::general_purpose::STANDARD
                    .decode(v)
                    .map_err(|e| HelixError::validation(format!("storage val b64: {e}")))?;
                map.insert(kb, vb);
            }
        }
        let signer: SignatureKeyPair = serde_json::from_slice(&dump.signer_json)
            .map_err(|e| HelixError::validation(format!("signer deser: {e}")))?;
        signer
            .store(provider.storage())
            .map_err(|e| HelixError::internal(format!("restore signer store: {e:?}")))?;
        let credential = BasicCredential::new(dump.identity);
        let credential_with_key = CredentialWithKey {
            credential: credential.into(),
            signature_key: signer.to_public_vec().into(),
        };
        self.users.lock().insert(
            user_key.to_string(),
            UserMls {
                provider,
                signer,
                credential: credential_with_key,
            },
        );
        Ok(())
    }

    pub fn has_user(&self, user_key: &str) -> bool {
        self.users.lock().contains_key(user_key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserDump {
    version: u32,
    label: String,
    /// Serialized SignatureKeyPair (serde).
    signer_json: Vec<u8>,
    identity: Vec<u8>,
    storage: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlsIdentityOut {
    pub user_key: String,
    pub label: String,
    pub signature_public_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlsGroupInfoOut {
    pub group_id: String,
    pub epoch: u64,
    pub members: Vec<String>,
    pub member_count: usize,
    pub exported_secret_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlsAddOut {
    pub commit_tls_b64: String,
    pub welcome_tls_b64: String,
    pub epoch: u64,
    pub members: Vec<String>,
    pub exported_secret_b64: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alice_bob_roundtrip() {
        let eng = MlsEngine::new();
        eng.ensure_identity("alice", "a").unwrap();
        eng.ensure_identity("bob", "b").unwrap();
        let gid = "00000000-0000-0000-0000-000000000000".to_string();
        eng.create_group("alice", &gid).unwrap();
        let kp = eng.create_key_package("bob").unwrap();
        let add = eng.add_member("alice", &gid, &kp).unwrap();
        let welcome = base64::engine::general_purpose::STANDARD
            .decode(&add.welcome_tls_b64)
            .unwrap();
        let joined = eng.join_with_welcome("bob", &welcome, None).unwrap();
        assert_eq!(joined.member_count, 2);
        assert_eq!(
            eng.export_group_secret("alice", &gid).unwrap(),
            eng.export_group_secret("bob", &gid).unwrap()
        );
        let msg = eng
            .create_app_message("alice", &gid, b"hello-sealed")
            .unwrap();
        let plain = eng.process_app_message("bob", &gid, &msg).unwrap();
        assert_eq!(plain.as_deref(), Some(b"hello-sealed".as_slice()));
    }

    #[test]
    fn durable_blob_survives_engine_restart() {
        let eng = MlsEngine::new();
        eng.ensure_identity("alice", "a").unwrap();
        eng.ensure_identity("bob", "b").unwrap();
        let gid = "doc-durable-1".to_string();
        eng.create_group("alice", &gid).unwrap();
        let kp = eng.create_key_package("bob").unwrap();
        let add = eng.add_member("alice", &gid, &kp).unwrap();
        let welcome = base64::engine::general_purpose::STANDARD
            .decode(&add.welcome_tls_b64)
            .unwrap();
        eng.join_with_welcome("bob", &welcome, None).unwrap();
        let sec = eng.export_group_secret("alice", &gid).unwrap();
        let alice_blob = eng.export_user_blob("alice").unwrap();
        let bob_blob = eng.export_user_blob("bob").unwrap();

        // Fresh engine simulates process restart
        let eng2 = MlsEngine::new();
        eng2.import_user_blob("alice", &alice_blob).unwrap();
        eng2.import_user_blob("bob", &bob_blob).unwrap();
        assert_eq!(eng2.export_group_secret("alice", &gid).unwrap(), sec);
        assert_eq!(eng2.export_group_secret("bob", &gid).unwrap(), sec);
        let msg = eng2
            .create_app_message("alice", &gid, b"after-restart")
            .unwrap();
        let plain = eng2.process_app_message("bob", &gid, &msg).unwrap();
        assert_eq!(plain.as_deref(), Some(b"after-restart".as_slice()));
    }
}
