use async_dup::Arc;
use bitcode::{Decode, Encode};
use camino::Utf8PathBuf;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::{RustFfiError, RustFfiResult, StorageError};

pub struct AppStorage {
    store: Arc<Database>,
}

impl AppStorage {
    pub async fn init(path: Utf8PathBuf) -> Result<Self, redb::Error> {
        blocking::unblock(move || {
            let store = Arc::new(Database::create(path)?);

            Ok(Self { store })
        })
        .await
    }

    pub(crate) async fn set(
        &self,
        table: TableDefinition<'static, &[u8], Vec<u8>>,
        key: impl AsRef<[u8]>,
        value: impl Encode + Decode<'static> + Send + Sync + 'static,
    ) -> RustFfiResult<()> {
        let key = key.as_ref().to_vec();

        let store = self.store.clone();

        blocking::unblock(move || {
            let write_txn = store.begin_write()?;

            {
                let mut table = write_txn.open_table(table)?;
                table.insert(key.as_slice(), bitcode::encode(&value))?;
            }

            write_txn.commit()?;

            Ok::<(), redb::Error>(())
        })
        .await?;

        Ok(())
    }

    pub(crate) async fn set_many(
        &self,
        table: TableDefinition<'static, &[u8], Vec<u8>>,
        key_values: Vec<(
            impl AsRef<[u8]> + 'static + Send + Sync,
            impl Encode + Decode<'static> + Send + Sync + 'static,
        )>,
    ) -> RustFfiResult<()> {
        let store = self.store.clone();

        blocking::unblock(move || {
            let write_txn = store.begin_write()?;

            {
                let mut table = write_txn.open_table(table)?;

                for (key, value) in key_values {
                    let key = key.as_ref().to_vec();

                    {
                        table.insert(key.as_slice(), bitcode::encode(&value))?;
                    }
                }
            }

            write_txn.commit()?;

            Ok::<(), redb::Error>(())
        })
        .await?;

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub(crate) async fn set_many_with_tables(
        &self,
        table_key_values: Vec<(
            TableDefinition<'static, &[u8], Vec<u8>>,
            impl AsRef<[u8]> + 'static + Send + Sync,
            impl Encode + Decode<'static> + Send + Sync + 'static,
        )>,
    ) -> RustFfiResult<()> {
        let store = self.store.clone();

        blocking::unblock(move || {
            let write_txn = store.begin_write()?;

            for (table, key, value) in table_key_values {
                let key = key.as_ref().to_vec();

                {
                    let mut table = write_txn.open_table(table)?;
                    table.insert(key.as_slice(), bitcode::encode(&value))?;
                }
            }

            write_txn.commit()?;

            Ok::<(), redb::Error>(())
        })
        .await?;

        Ok(())
    }

    pub(crate) async fn insert_if_not_exist(
        &self,
        table: TableDefinition<'static, &[u8], Vec<u8>>,
        key: impl AsRef<[u8]>,
        value: impl Encode + Decode<'static> + Send + Sync + 'static,
    ) -> RustFfiResult<()> {
        let key = key.as_ref().to_vec();

        let store = self.store.clone();

        blocking::unblock(move || {
            let write_txn = store.begin_write()?;
            {
                let mut table = write_txn.open_table(table)?;

                if table.get(key.as_slice())?.is_some() {
                    return Err(RustFfiError::Storage(StorageError::KeyAlreadyExists));
                }

                table.insert(key.as_slice(), bitcode::encode(&value))?;
            }
            write_txn.commit()?;

            Ok::<(), RustFfiError>(())
        })
        .await
    }

    pub(crate) async fn update<F: Encode + Decode<'static> + Send + Sync + 'static>(
        &self,
        table: TableDefinition<'static, &[u8], Vec<u8>>,
        key: impl AsRef<[u8]>,
        callback: impl FnOnce(&[u8]) -> RustFfiResult<F> + Send + 'static,
    ) -> RustFfiResult<()> {
        let key = key.as_ref().to_vec();

        let store = self.store.clone();

        blocking::unblock(move || {
            let write_txn = store.begin_write()?;
            {
                let mut table = write_txn.open_table(table)?;

                let mut value = table
                    .get_mut(key.as_slice())?
                    .ok_or(RustFfiError::Storage(StorageError::KeyToUpdateNotFound))?;

                let execute = callback(value.value().as_slice())?;
                value.insert(bitcode::encode(&execute))?;
            }

            write_txn.commit()?;

            Ok::<(), RustFfiError>(())
        })
        .await
    }

    pub(crate) async fn get(
        &self,
        table: TableDefinition<'static, &[u8], Vec<u8>>,
        key: impl AsRef<[u8]>,
    ) -> RustFfiResult<Option<Vec<u8>>> {
        let key = key.as_ref().to_vec();

        let store = self.store.clone();

        let fetched = blocking::unblock(move || {
            let read_txn = store.begin_read()?;
            let table = read_txn.open_table(table)?;

            let fetched = table
                .get(key.as_slice())?
                .map(|inner_value| inner_value.value());

            Ok::<_, redb::Error>(fetched)
        })
        .await?;

        Ok(fetched)
    }
}
