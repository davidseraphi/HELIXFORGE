# Resource ACL + governance (Core)

Products call these HelixCore APIs instead of inventing per-product ACLs.

## Resource ACL

```
POST /v1/acl/{type}/{id}          # grant  { principal_kind, principal_id, permissions[] }
GET  /v1/acl/{type}/{id}          # list grants
POST /v1/acl/{type}/{id}/revoke   # revoke
GET  /v1/acl/{type}/{id}/check?permission=read
```

`principal_kind`: `user` | `api_key` | `role` | `tenant`  
`permissions`: `read` | `write` | `delete` | `share` | `admin`

### Product usage (Rust)

```rust
state.clients.acl.as_ref().unwrap()
  .require(&principal, "document", &doc_id, AclPermission::Write)
  .await?;
```

On create, grant owner:

```rust
acl.grant(tenant, "document", &id, "user", &user_id, &[AclPermission::Admin], Some(...)).await?;
```

## Retention / legal hold / purpose

```
POST /v1/governance/retention
GET  /v1/governance/retention
POST /v1/governance/holds
POST /v1/governance/holds/{id}/release
POST /v1/governance/purpose
GET  /v1/governance/can-delete/{type}/{id}
```

Before product `DELETE`, call `can-delete` or `GovernanceRepo::can_delete` — blocks when hold active or retention requires review.

## Multi-region

```
GET  /v1/regions
POST /v1/regions/{code}/status   # Platform
```

Service region = `HELIX_DATA_RESIDENCY`. Use `RegionRepo::assert_write_allowed` on mutating product routes when multi-region is live.
