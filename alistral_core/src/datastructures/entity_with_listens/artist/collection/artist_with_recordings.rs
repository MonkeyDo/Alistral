use itertools::Itertools as _;
use musicbrainz_db_lite::models::musicbrainz::artist::Artist;
use musicbrainz_db_lite::models::musicbrainz::recording::Recording;

use crate::database::fetching::recordings::fetch_recordings_as_complete;
use crate::datastructures::entity_with_listens::artist::artist_with_recordings::ArtistWithRecordings;
use crate::datastructures::entity_with_listens::collection::EntityWithListensCollection;
use crate::datastructures::entity_with_listens::recording::collection::RecordingWithListensCollection;
use crate::datastructures::listen_collection::ListenCollection;

pub type ArtistWithRecordingsCollection =
    EntityWithListensCollection<Artist, RecordingWithListensCollection>;

impl ArtistWithRecordingsCollection {
    pub async fn from_listencollection(
        conn: &mut sqlx::SqliteConnection,
        listens: ListenCollection,
    ) -> Result<Self, crate::Error> {
        // Convert Recordings
        let recordings =
            RecordingWithListensCollection::from_listencollection(conn, listens).await?;

        let recording_refs = recordings.iter_entities().collect_vec();
        fetch_recordings_as_complete(conn, &recording_refs).await?;

        // Load artists
        let results = Recording::get_artist_from_credits_as_batch(conn, &recording_refs).await?;

        // Convert artists
        let mut out = Self::new();

        for (_, (recording, artists)) in results {
            for artist in artists {
                let recording = recordings
                    .0
                    .get(&recording.id)
                    .expect("The artist has been fetched from the recording, so it should be there")
                    .clone();

                let artist_with_recordings = ArtistWithRecordings {
                    entity: artist,
                    listens: recording.into(),
                };

                out.insert_or_merge_entity(artist_with_recordings);
            }
        }

        Ok(out)
    }
}
