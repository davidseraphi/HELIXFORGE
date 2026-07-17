//! HelixCapital accounts + double-entry journal persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_core::ids::TenantId;
use shared_core::{HelixError, HelixResult};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub kind: String,
    pub currency: String,
    pub balance_cents: i64,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLine {
    pub id: Uuid,
    pub journal_id: Uuid,
    pub account_id: Uuid,
    pub side: String,
    pub amount_cents: i64,
    pub memo: String,
    pub is_reversal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub memo: String,
    pub status: String,
    pub currency: String,
    pub metadata: serde_json::Value,
    pub lines: Vec<JournalLine>,
    pub posted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub voided_at: Option<DateTime<Utc>>,
    pub void_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TrialBalanceRow {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub kind: String,
    pub currency: String,
    pub balance_cents: i64,
}

#[derive(Debug, Clone)]
pub struct JournalLineInput {
    pub account_id: Uuid,
    pub side: String,
    pub amount_cents: i64,
    pub memo: String,
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    tenant_id: Uuid,
    code: String,
    name: String,
    kind: String,
    currency: String,
    balance_cents: i64,
    status: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl AccountRow {
    fn into_account(self) -> Account {
        Account {
            id: self.id,
            tenant_id: TenantId::from_uuid(self.tenant_id),
            code: self.code,
            name: self.name,
            kind: self.kind,
            currency: self.currency,
            balance_cents: self.balance_cents,
            status: self.status,
            metadata: self.metadata,
            created_at: self.created_at,
        }
    }
}

const ACCOUNT_SELECT: &str = r#"
    SELECT id, tenant_id, code, name, kind, currency, balance_cents, status, metadata, created_at
    FROM capital.accounts
"#;

#[derive(Clone)]
pub struct CapitalRepo {
    pool: PgPool,
}

impl CapitalRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_accounts(&self, tenant_id: TenantId) -> HelixResult<Vec<Account>> {
        let rows: Vec<AccountRow> = sqlx::query_as(&format!(
            "{ACCOUNT_SELECT} WHERE tenant_id = $1 AND deleted_at IS NULL ORDER BY code ASC"
        ))
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital list accounts: {e}")))?;
        Ok(rows.into_iter().map(AccountRow::into_account).collect())
    }

    pub async fn create_account(
        &self,
        tenant_id: TenantId,
        code: &str,
        name: &str,
        kind: &str,
        currency: &str,
        metadata: serde_json::Value,
    ) -> HelixResult<Account> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let kind = if kind.trim().is_empty() {
            "asset"
        } else {
            kind.trim()
        };
        let currency = if currency.trim().is_empty() {
            "USD"
        } else {
            currency.trim()
        };
        sqlx::query(
            r#"
            INSERT INTO capital.accounts
                (id, tenant_id, code, name, kind, currency, balance_cents, status, metadata, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,0,'open',$7,$8,$8)
            "#,
        )
        .bind(id)
        .bind(tenant_id.as_uuid())
        .bind(code)
        .bind(name)
        .bind(kind)
        .bind(currency)
        .bind(&metadata)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital create account: {e}")))?;
        Ok(Account {
            id,
            tenant_id,
            code: code.into(),
            name: name.into(),
            kind: kind.into(),
            currency: currency.into(),
            balance_cents: 0,
            status: "open".into(),
            metadata,
            created_at,
        })
    }

    pub async fn get_account(
        &self,
        tenant_id: TenantId,
        account_id: Uuid,
    ) -> HelixResult<Option<Account>> {
        let row: Option<AccountRow> = sqlx::query_as(&format!(
            "{ACCOUNT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital get account: {e}")))?;
        Ok(row.map(AccountRow::into_account))
    }

    pub async fn update_account(
        &self,
        tenant_id: TenantId,
        account_id: Uuid,
        name: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> HelixResult<Account> {
        let mut builder = sqlx::QueryBuilder::new("UPDATE capital.accounts SET updated_at = ");
        builder.push_bind(Utc::now());

        if let Some(n) = name {
            builder.push(", name = ");
            builder.push_bind(n);
        }
        if let Some(m) = metadata {
            builder.push(", metadata = ");
            builder.push_bind(m);
        }
        builder.push(" WHERE tenant_id = ");
        builder.push_bind(tenant_id.as_uuid());
        builder.push(" AND id = ");
        builder.push_bind(account_id);
        builder.push(" AND deleted_at IS NULL");
        builder.push(" RETURNING id, tenant_id, code, name, kind, currency, balance_cents, status, metadata, created_at");

        let row: Option<AccountRow> = builder
            .build_query_as::<AccountRow>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| HelixError::dependency(format!("capital update account: {e}")))?;

        row.map(AccountRow::into_account)
            .ok_or_else(|| HelixError::not_found("account not found"))
    }

    pub async fn close_account(
        &self,
        tenant_id: TenantId,
        account_id: Uuid,
    ) -> HelixResult<Account> {
        let closed_at = Utc::now();
        let row: Option<AccountRow> = sqlx::query_as(&format!(
            "{ACCOUNT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital close account fetch: {e}")))?;

        let account = row
            .map(AccountRow::into_account)
            .ok_or_else(|| HelixError::not_found("account not found"))?;

        if account.status != "open" {
            return Err(HelixError::validation(format!(
                "account {} is not open",
                account.code
            )));
        }
        if account.balance_cents != 0 {
            return Err(HelixError::validation(format!(
                "account {} balance must be zero to close",
                account.code
            )));
        }

        let row: Option<AccountRow> = sqlx::query_as(
            r#"
            UPDATE capital.accounts
            SET status = 'closed', closed_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            RETURNING id, tenant_id, code, name, kind, currency, balance_cents, status, metadata, created_at
            "#,
        )
        .bind(closed_at)
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital close account: {e}")))?;

        row.map(AccountRow::into_account)
            .ok_or_else(|| HelixError::not_found("account not found"))
    }

    pub async fn reopen_account(
        &self,
        tenant_id: TenantId,
        account_id: Uuid,
    ) -> HelixResult<Account> {
        let reopened_at = Utc::now();
        let row: Option<AccountRow> = sqlx::query_as(
            r#"
            UPDATE capital.accounts
            SET status = 'open', closed_at = NULL, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL AND status = 'closed'
            RETURNING id, tenant_id, code, name, kind, currency, balance_cents, status, metadata, created_at
            "#,
        )
        .bind(reopened_at)
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital reopen account: {e}")))?;

        row.map(AccountRow::into_account)
            .ok_or_else(|| HelixError::not_found("account not found or not closed"))
    }

    pub async fn soft_delete_account(
        &self,
        tenant_id: TenantId,
        account_id: Uuid,
    ) -> HelixResult<Account> {
        let row: Option<AccountRow> = sqlx::query_as(&format!(
            "{ACCOUNT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL"
        ))
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital soft-delete account fetch: {e}")))?;

        let account = row
            .map(AccountRow::into_account)
            .ok_or_else(|| HelixError::not_found("account not found"))?;

        if account.status == "deleted" {
            return Err(HelixError::validation(format!(
                "account {} is already deleted",
                account.code
            )));
        }

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM capital.journal_lines WHERE tenant_id = $1 AND account_id = $2",
        )
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital check account lines: {e}")))?;

        if count > 0 {
            return Err(HelixError::validation(format!(
                "account {} has journal entries and cannot be deleted",
                account.code
            )));
        }

        let deleted_at = Utc::now();
        let row: Option<AccountRow> = sqlx::query_as(
            r#"
            UPDATE capital.accounts
            SET status = 'deleted', deleted_at = $1, updated_at = $1
            WHERE tenant_id = $2 AND id = $3 AND deleted_at IS NULL
            RETURNING id, tenant_id, code, name, kind, currency, balance_cents, status, metadata, created_at
            "#,
        )
        .bind(deleted_at)
        .bind(tenant_id.as_uuid())
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital soft-delete account: {e}")))?;

        row.map(AccountRow::into_account)
            .ok_or_else(|| HelixError::not_found("account not found"))
    }

    pub async fn list_journals(&self, tenant_id: TenantId) -> HelixResult<Vec<Journal>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            memo: String,
            status: String,
            currency: String,
            metadata: serde_json::Value,
            posted_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
            voided_at: Option<DateTime<Utc>>,
            void_reason: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, memo, status, currency, metadata, posted_at, created_at, voided_at, void_reason
            FROM capital.journals
            WHERE tenant_id = $1
            ORDER BY posted_at DESC
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital list journals: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let lines = self.load_lines(r.id).await?;
            out.push(Journal {
                id: r.id,
                tenant_id: TenantId::from_uuid(r.tenant_id),
                memo: r.memo,
                status: r.status,
                currency: r.currency,
                metadata: r.metadata,
                lines,
                posted_at: r.posted_at,
                created_at: r.created_at,
                voided_at: r.voided_at,
                void_reason: r.void_reason,
            });
        }
        Ok(out)
    }

    pub async fn get_journal(
        &self,
        tenant_id: TenantId,
        journal_id: Uuid,
    ) -> HelixResult<Option<Journal>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            tenant_id: Uuid,
            memo: String,
            status: String,
            currency: String,
            metadata: serde_json::Value,
            posted_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
            voided_at: Option<DateTime<Utc>>,
            void_reason: String,
        }
        let row: Option<Row> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, memo, status, currency, metadata, posted_at, created_at, voided_at, void_reason
            FROM capital.journals
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(journal_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital get journal: {e}")))?;
        let Some(r) = row else {
            return Ok(None);
        };
        let lines = self.load_lines(r.id).await?;
        Ok(Some(Journal {
            id: r.id,
            tenant_id: TenantId::from_uuid(r.tenant_id),
            memo: r.memo,
            status: r.status,
            currency: r.currency,
            metadata: r.metadata,
            lines,
            posted_at: r.posted_at,
            created_at: r.created_at,
            voided_at: r.voided_at,
            void_reason: r.void_reason,
        }))
    }

    /// Post a balanced journal. Debits must equal credits; account balances update in-txn.
    /// Balance sign: debit increases, credit decreases (simple cash-basis asset ledger).
    pub async fn post_journal(
        &self,
        tenant_id: TenantId,
        memo: &str,
        currency: &str,
        lines: &[JournalLineInput],
        metadata: serde_json::Value,
    ) -> HelixResult<Journal> {
        if lines.len() < 2 {
            return Err(HelixError::validation(
                "journal requires at least two lines",
            ));
        }

        let mut debit_total: i64 = 0;
        let mut credit_total: i64 = 0;
        for line in lines {
            let side = line.side.trim().to_ascii_lowercase();
            if side != "debit" && side != "credit" {
                return Err(HelixError::validation("side must be debit or credit"));
            }
            if line.amount_cents <= 0 {
                return Err(HelixError::validation("amount_cents must be > 0"));
            }
            if side == "debit" {
                debit_total = debit_total
                    .checked_add(line.amount_cents)
                    .ok_or_else(|| HelixError::validation("debit total overflow"))?;
            } else {
                credit_total = credit_total
                    .checked_add(line.amount_cents)
                    .ok_or_else(|| HelixError::validation("credit total overflow"))?;
            }
        }
        if debit_total != credit_total {
            return Err(HelixError::validation(format!(
                "unbalanced journal: debits={debit_total} credits={credit_total}"
            )));
        }

        let currency = if currency.trim().is_empty() {
            "USD"
        } else {
            currency.trim()
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("capital begin: {e}")))?;

        let journal_id = Uuid::now_v7();
        let posted_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO capital.journals
                (id, tenant_id, memo, status, currency, metadata, posted_at, created_at)
            VALUES ($1,$2,$3,'posted',$4,$5,$6,$6)
            "#,
        )
        .bind(journal_id)
        .bind(tenant_id.as_uuid())
        .bind(memo)
        .bind(currency)
        .bind(&metadata)
        .bind(posted_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("capital insert journal: {e}")))?;

        let mut out_lines = Vec::with_capacity(lines.len());
        for line in lines {
            let side = line.side.trim().to_ascii_lowercase();
            let acct: Option<AccountRow> = sqlx::query_as(&format!(
                "{ACCOUNT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE"
            ))
            .bind(tenant_id.as_uuid())
            .bind(line.account_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital lock account: {e}")))?;

            let account = acct
                .map(AccountRow::into_account)
                .ok_or_else(|| HelixError::not_found(format!("account {}", line.account_id)))?;
            if account.status != "open" {
                return Err(HelixError::validation(format!(
                    "account {} is not open",
                    account.code
                )));
            }

            let delta = if side == "debit" {
                line.amount_cents
            } else {
                -line.amount_cents
            };

            sqlx::query(
                r#"
                UPDATE capital.accounts
                SET balance_cents = balance_cents + $1, updated_at = $2
                WHERE id = $3 AND tenant_id = $4
                "#,
            )
            .bind(delta)
            .bind(posted_at)
            .bind(account.id)
            .bind(tenant_id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital update balance: {e}")))?;

            let line_id = Uuid::now_v7();
            sqlx::query(
                r#"
                INSERT INTO capital.journal_lines
                    (id, journal_id, tenant_id, account_id, side, amount_cents, memo, is_reversal)
                VALUES ($1,$2,$3,$4,$5,$6,$7,false)
                "#,
            )
            .bind(line_id)
            .bind(journal_id)
            .bind(tenant_id.as_uuid())
            .bind(account.id)
            .bind(&side)
            .bind(line.amount_cents)
            .bind(&line.memo)
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital insert line: {e}")))?;

            out_lines.push(JournalLine {
                id: line_id,
                journal_id,
                account_id: account.id,
                side,
                amount_cents: line.amount_cents,
                memo: line.memo.clone(),
                is_reversal: false,
            });
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("capital commit journal: {e}")))?;

        Ok(Journal {
            id: journal_id,
            tenant_id,
            memo: memo.into(),
            status: "posted".into(),
            currency: currency.into(),
            metadata,
            lines: out_lines,
            posted_at,
            created_at: posted_at,
            voided_at: None,
            void_reason: String::new(),
        })
    }

    pub async fn void_journal(
        &self,
        tenant_id: TenantId,
        journal_id: Uuid,
        reason: Option<String>,
    ) -> HelixResult<Journal> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("capital void begin: {e}")))?;

        #[derive(sqlx::FromRow)]
        #[allow(dead_code)]
        struct JournalRow {
            id: Uuid,
            tenant_id: Uuid,
            memo: String,
            status: String,
            currency: String,
            metadata: serde_json::Value,
            posted_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
            voided_at: Option<DateTime<Utc>>,
            void_reason: String,
        }

        let row: Option<JournalRow> = sqlx::query_as(
            r#"
            SELECT id, tenant_id, memo, status, currency, metadata, posted_at, created_at, voided_at, void_reason
            FROM capital.journals
            WHERE tenant_id = $1 AND id = $2
            FOR UPDATE
            "#,
        )
        .bind(tenant_id.as_uuid())
        .bind(journal_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("capital lock journal: {e}")))?;

        let journal = row.ok_or_else(|| HelixError::not_found("journal not found"))?;
        if journal.status != "posted" {
            return Err(HelixError::validation(format!(
                "cannot void journal with status {}",
                journal.status
            )));
        }

        let lines = self.load_lines_in_tx(journal.id, &mut tx).await?;
        let voided_at = Utc::now();
        let void_reason = reason.unwrap_or_default();

        for line in &lines {
            let acct: Option<AccountRow> = sqlx::query_as(&format!(
                "{ACCOUNT_SELECT} WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE"
            ))
            .bind(tenant_id.as_uuid())
            .bind(line.account_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital void lock account: {e}")))?;

            let account = acct
                .map(AccountRow::into_account)
                .ok_or_else(|| HelixError::not_found(format!("account {}", line.account_id)))?;
            if account.status != "open" {
                return Err(HelixError::validation(format!(
                    "account {} is not open; cannot void against a closed account",
                    account.code
                )));
            }

            let reversal_delta = if line.side == "debit" {
                -line.amount_cents
            } else {
                line.amount_cents
            };

            sqlx::query(
                r#"
                UPDATE capital.accounts
                SET balance_cents = balance_cents + $1, updated_at = $2
                WHERE id = $3 AND tenant_id = $4
                "#,
            )
            .bind(reversal_delta)
            .bind(voided_at)
            .bind(account.id)
            .bind(tenant_id.as_uuid())
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital void update balance: {e}")))?;

            let reversal_side = if line.side == "debit" {
                "credit"
            } else {
                "debit"
            };
            let reversal_id = Uuid::now_v7();
            sqlx::query(
                r#"
                INSERT INTO capital.journal_lines
                    (id, journal_id, tenant_id, account_id, side, amount_cents, memo, is_reversal)
                VALUES ($1,$2,$3,$4,$5,$6,$7,true)
                "#,
            )
            .bind(reversal_id)
            .bind(journal.id)
            .bind(tenant_id.as_uuid())
            .bind(account.id)
            .bind(reversal_side)
            .bind(line.amount_cents)
            .bind(format!("void: {}", line.memo))
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital insert reversal line: {e}")))?;
        }

        sqlx::query(
            r#"
            UPDATE capital.journals
            SET status = 'voided', voided_at = $1, void_reason = $2
            WHERE tenant_id = $3 AND id = $4
            "#,
        )
        .bind(voided_at)
        .bind(&void_reason)
        .bind(tenant_id.as_uuid())
        .bind(journal_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("capital void journal: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("capital commit void: {e}")))?;

        self.get_journal(tenant_id, journal_id)
            .await?
            .ok_or_else(|| HelixError::internal("journal missing after void"))
    }

    pub async fn get_trial_balance(
        &self,
        tenant_id: TenantId,
    ) -> HelixResult<Vec<TrialBalanceRow>> {
        let rows: Vec<TrialBalanceRow> = sqlx::query_as(
            r#"
            SELECT id, code, name, kind, currency, balance_cents
            FROM capital.accounts
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY kind, code
            "#,
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital trial balance: {e}")))?;
        Ok(rows)
    }

    pub async fn record_balance_snapshot(&self, tenant_id: TenantId) -> HelixResult<u64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| HelixError::dependency(format!("capital snapshot begin: {e}")))?;

        let captured_at = Utc::now();
        let rows: Vec<(Uuid, i64)> = sqlx::query_as(
            "SELECT id, balance_cents FROM capital.accounts WHERE tenant_id = $1 AND deleted_at IS NULL",
        )
        .bind(tenant_id.as_uuid())
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| HelixError::dependency(format!("capital snapshot fetch: {e}")))?;

        let count = rows.len();
        for (account_id, balance_cents) in rows {
            sqlx::query(
                r#"
                INSERT INTO capital.account_balance_history
                    (id, tenant_id, account_id, balance_cents, captured_at)
                VALUES ($1,$2,$3,$4,$5)
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(tenant_id.as_uuid())
            .bind(account_id)
            .bind(balance_cents)
            .bind(captured_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| HelixError::dependency(format!("capital snapshot insert: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| HelixError::dependency(format!("capital commit snapshot: {e}")))?;

        Ok(count as u64)
    }

    async fn load_lines(&self, journal_id: Uuid) -> HelixResult<Vec<JournalLine>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            journal_id: Uuid,
            account_id: Uuid,
            side: String,
            amount_cents: i64,
            memo: String,
            is_reversal: bool,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, journal_id, account_id, side, amount_cents, memo, is_reversal
            FROM capital.journal_lines
            WHERE journal_id = $1
            ORDER BY side, amount_cents DESC
            "#,
        )
        .bind(journal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HelixError::dependency(format!("capital load lines: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| JournalLine {
                id: r.id,
                journal_id: r.journal_id,
                account_id: r.account_id,
                side: r.side,
                amount_cents: r.amount_cents,
                memo: r.memo,
                is_reversal: r.is_reversal,
            })
            .collect())
    }

    async fn load_lines_in_tx(
        &self,
        journal_id: Uuid,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> HelixResult<Vec<JournalLine>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            journal_id: Uuid,
            account_id: Uuid,
            side: String,
            amount_cents: i64,
            memo: String,
            is_reversal: bool,
        }
        let rows: Vec<Row> = sqlx::query_as(
            r#"
            SELECT id, journal_id, account_id, side, amount_cents, memo, is_reversal
            FROM capital.journal_lines
            WHERE journal_id = $1 AND is_reversal = false
            ORDER BY side, amount_cents DESC
            "#,
        )
        .bind(journal_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| HelixError::dependency(format!("capital load lines in tx: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| JournalLine {
                id: r.id,
                journal_id: r.journal_id,
                account_id: r.account_id,
                side: r.side,
                amount_cents: r.amount_cents,
                memo: r.memo,
                is_reversal: r.is_reversal,
            })
            .collect())
    }
}
