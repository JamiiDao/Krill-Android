use redb::TableDefinition;

use crate::{AppStorage, RustFfiResult, StoredFrostParticipateMessage};

impl AppStorage {
    pub(crate) const ACTIVITIES_TABLE: TableDefinition<'static, &[u8], Vec<u8>> =
        TableDefinition::new("Activities");

    pub async fn set_activity(
        &self,
        to_storage: StoredFrostParticipateMessage,
    ) -> RustFfiResult<()> {
        self.set(Self::ACTIVITIES_TABLE, to_storage.activity_id, to_storage)
            .await
    }
}
