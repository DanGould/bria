use crate::{
    account::{keys::*, *},
    ledger::Ledger,
};
use tracing::instrument;

use super::{error::*, keys::*};

const BOOTSTRAP_KEY_NAME: &str = "admin_bootstrap_key";

pub struct AdminApp {
    keys: AdminApiKeys,
    accounts: Accounts,
    account_keys: AccountApiKeys,
    ledger: Ledger,
    pool: sqlx::PgPool,
}

impl AdminApp {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            keys: AdminApiKeys::new(&pool),
            accounts: Accounts::new(&pool),
            account_keys: AccountApiKeys::new(&pool),
            ledger: Ledger::new(&pool),
            pool,
        }
    }
}

impl AdminApp {
    #[instrument(name = "admin_app.bootstrap", skip(self), err)]
    pub async fn bootstrap(&self) -> Result<AdminApiKey, AdminApiError> {
        self.keys.create(BOOTSTRAP_KEY_NAME.to_string()).await
    }

    #[instrument(name = "admin_app.authenticate", skip(self), err)]
    pub async fn authenticate(&self, key: &str) -> Result<(), AdminApiError> {
        self.keys.find_by_key(key).await?;
        Ok(())
    }

    #[instrument(name = "admin_app.create_account", skip(self), err)]
    pub async fn create_account(
        &self,
        account_name: String,
    ) -> Result<AccountApiKey, AdminApiError> {
        let mut tx = self.pool.begin().await?;
        let account = self
            .accounts
            .create_in_tx(&mut tx, account_name.clone())
            .await?;
        self.ledger
            .create_journal_for_account(&mut tx, account.id, account.name.clone())
            .await?;
        let keys = self
            .account_keys
            .create_in_tx(&mut tx, account_name, account.id)
            .await?;
        tx.commit().await?;
        Ok(keys)
    }

    #[instrument(name = "admin_app.list_accounts", skip(self), err)]
    pub async fn list_accounts(&self) -> Result<Vec<Account>, AdminApiError> {
        self.accounts.list().await
    }
}
