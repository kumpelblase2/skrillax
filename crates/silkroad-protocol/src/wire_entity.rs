use skrillax_serde::SerdeContext;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The entity category that determines an entity-dependent packet layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireEntityKind {
    Monster,
    Npc,
    Other,
}

/// Whether a runtime entity is currently visible on this connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireEntityStatus {
    Active,
    Retired,
}

/// Wire metadata retained independently from the corresponding ECS entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireEntityRecord {
    pub ref_id: u32,
    pub kind: WireEntityKind,
    pub status: WireEntityStatus,
}

/// Connection-scoped knowledge needed to decode packets keyed by runtime IDs.
///
/// Retired entries remain available so packets delayed across a despawn can
/// still be decoded. A later spawn of the same ID replaces and reactivates the
/// entry.
#[derive(Clone, Default)]
pub struct WireEntityCatalog {
    entries: Arc<RwLock<HashMap<u32, WireEntityRecord>>>,
}

impl WireEntityCatalog {
    /// Creates and installs the catalog required by a connection's serialization context.
    ///
    /// Call this once while constructing the connection, before either stream
    /// half begins processing packets.
    pub fn install(ctx: &SerdeContext) -> Self {
        let catalog = Self::default();
        ctx.set(catalog.clone());
        catalog
    }

    /// Returns the catalog installed for this serialization context.
    ///
    /// A missing catalog is a connection setup error; catalogs are deliberately
    /// installed eagerly rather than created while processing a packet.
    pub fn for_context(ctx: &SerdeContext) -> Self {
        ctx.get::<Self>()
            .expect("WireEntityCatalog must be installed before processing packets")
    }

    pub fn lookup(&self, unique_id: u32) -> Option<WireEntityRecord> {
        self.entries
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&unique_id)
            .copied()
    }

    pub(crate) fn record_spawn(&self, unique_id: u32, ref_id: u32, kind: WireEntityKind) {
        self.entries
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                unique_id,
                WireEntityRecord {
                    ref_id,
                    kind,
                    status: WireEntityStatus::Active,
                },
            );
    }

    pub(crate) fn retire(&self, unique_id: u32) {
        if let Some(record) = self
            .entries
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get_mut(&unique_id)
        {
            record.status = WireEntityStatus::Retired;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "WireEntityCatalog must be installed")]
    fn missing_catalog_is_a_connection_setup_error() {
        WireEntityCatalog::for_context(&SerdeContext::default());
    }

    #[test]
    fn cloned_contexts_share_a_catalog() {
        let context = SerdeContext::default();
        let cloned_context = context.clone();
        let catalog = WireEntityCatalog::install(&context);

        catalog.record_spawn(12, 34, WireEntityKind::Monster);

        assert_eq!(
            WireEntityCatalog::for_context(&cloned_context).lookup(12),
            Some(WireEntityRecord {
                ref_id: 34,
                kind: WireEntityKind::Monster,
                status: WireEntityStatus::Active,
            })
        );
    }

    #[test]
    fn separate_contexts_do_not_share_catalogs() {
        let first = SerdeContext::default();
        let second = SerdeContext::default();
        let first_catalog = WireEntityCatalog::install(&first);
        WireEntityCatalog::install(&second);

        first_catalog.record_spawn(12, 34, WireEntityKind::Npc);

        assert_eq!(WireEntityCatalog::for_context(&second).lookup(12), None);
    }

    #[test]
    fn one_catalog_can_be_installed_in_direction_local_contexts() {
        let server_to_client = SerdeContext::default();
        let client_to_server = SerdeContext::default();
        let catalog = WireEntityCatalog::default();
        server_to_client.set(catalog.clone());
        client_to_server.set(catalog);

        WireEntityCatalog::for_context(&server_to_client).record_spawn(12, 34, WireEntityKind::Npc);

        assert_eq!(
            WireEntityCatalog::for_context(&client_to_server).lookup(12),
            Some(WireEntityRecord {
                ref_id: 34,
                kind: WireEntityKind::Npc,
                status: WireEntityStatus::Active,
            })
        );
    }

    #[test]
    fn retirement_preserves_wire_metadata() {
        let catalog = WireEntityCatalog::default();
        catalog.record_spawn(12, 34, WireEntityKind::Monster);
        catalog.retire(12);

        assert_eq!(
            catalog.lookup(12),
            Some(WireEntityRecord {
                ref_id: 34,
                kind: WireEntityKind::Monster,
                status: WireEntityStatus::Retired,
            })
        );
    }
}
