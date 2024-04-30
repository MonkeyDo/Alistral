use crate::core::caching::entity_cache::EntityCache;
use crate::core::entity_traits::cached::Cached;
use crate::core::entity_traits::has_id::HasID;
use crate::core::entity_traits::insertable::{
    InsertableAs, InsertableWithExtras, IsAutoInsertableAs,
};
use crate::core::entity_traits::merge::UpdateCachedEntity;
use crate::models::data::entity_database::ENTITY_DATABASE;
use crate::models::data::musicbrainz::artist::Artist;
use crate::models::data::musicbrainz::HasMbid;
use musicbrainz_rs::entity::artist::Artist as ArtistMS;
use std::sync::Arc;

impl HasMbid for ArtistMS {
    fn get_mbid(&self) -> &str {
        &self.id
    }
}

impl UpdateCachedEntity for Artist {
    fn update_entity(self, new: Self) -> Self {
        new
    }
}

impl Cached<String> for Artist {
    fn get_cache() -> Arc<EntityCache<String, Self>>
    where
        Self: Sized,
    {
        ENTITY_DATABASE.artists()
    }
}

impl InsertableAs<String, Artist> for ArtistMS {
    async fn insert_into_cache_as(&self, key: String) -> color_eyre::Result<()> {
        Artist::get_cache().set(&key, self.clone().into()).await?;

        Ok(())
    }
}

impl InsertableWithExtras<String, Artist> for ArtistMS {
    async fn insert_with_relations(&self, key: String) -> color_eyre::Result<()> {
        Artist::get_cache().set(&key, self.clone().into()).await?;

        if let Some(recordings) = self.recordings.clone() {
            for item in recordings.iter() {
                item.insert_into_cache().await?;
            }
        }

        Ok(())
    }
}

impl HasID<String> for Artist {
    fn get_id(&self) -> String {
        self.id.to_string()
    }
}

impl HasID<String> for ArtistMS {
    fn get_id(&self) -> String {
        self.id.to_string()
    }
}