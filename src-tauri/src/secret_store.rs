use keyring::Entry;

use wealthvn_ai::SecretStore;

const USERNAME: &str = "default";

#[derive(Debug, Default)]
pub struct KeyringSecretStore;

impl SecretStore for KeyringSecretStore {
    fn set_secret(&self, service: &str, secret: &str) -> std::result::Result<(), wealthvn_ai::SecretStoreError> {
        let entry = entry_for(service)?;
        entry
            .set_password(secret)
            .map_err(|err| wealthvn_ai::SecretStoreError::AccessFailed(err.to_string()))
    }

    fn get_secret(&self, service: &str) -> std::result::Result<Option<String>, wealthvn_ai::SecretStoreError> {
        let entry = entry_for(service)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(wealthvn_ai::SecretStoreError::AccessFailed(err.to_string())),
        }
    }

    fn delete_secret(&self, service: &str) -> std::result::Result<(), wealthvn_ai::SecretStoreError> {
        let entry = entry_for(service)?;
        match entry.delete_password() {
            Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(wealthvn_ai::SecretStoreError::AccessFailed(err.to_string())),
        }
    }
}

fn entry_for(service: &str) -> std::result::Result<Entry, wealthvn_ai::SecretStoreError> {
    // Format the service ID for keyring - use the service directly as the keyring service name
    let service_id = format!("wealthvn_{}", service);
    Entry::new(&service_id, USERNAME).map_err(|err| wealthvn_ai::SecretStoreError::AccessFailed(err.to_string()))
}
