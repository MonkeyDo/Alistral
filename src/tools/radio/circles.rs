use async_fn_stream::try_fn_stream;
use futures::pin_mut;
use futures::Stream;
use futures::TryStreamExt;
use itertools::Itertools;
use musicbrainz_db_lite::models::musicbrainz::artist::Artist;
use musicbrainz_db_lite::models::musicbrainz::recording::Recording;
use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::database::get_db_client;
use crate::datastructures::radio::collector::RadioCollector;
use crate::datastructures::radio::seeders::listens::ListenSeeder;
use crate::models::data::musicbrainz::recording::mbid::RecordingMBID;
use crate::utils::playlist::PlaylistStub;
use crate::utils::println_cli;

pub async fn create_radio_mix(
    seeder: ListenSeeder,
    token: String,
    unlistened: bool,
    collector: RadioCollector,
) {
    let db = get_db_client().await;
    let conn = &mut *db
        .connection
        .acquire()
        .await
        .expect("Couldn't get a database connection");

    let username = seeder.username().clone();

    println_cli("[Seeding] Getting listens");
    let recordings_with_listens = seeder.seed(conn).await.expect("Couldn't find seed listens");

    let recordings = recordings_with_listens.iter_recordings().collect_vec();

    let radio = RadioCircle::new(unlistened);
    let radio_stream = radio.into_stream(conn, recordings);

    println_cli("[Finalising] Creating radio playlist");
    pin_mut!(radio_stream);
    let collected = collector
        .try_collect(radio_stream)
        .await
        .expect("Error while generating the playlist");

    PlaylistStub::new(
        "Radio: Circles".to_string(),
        Some(username.to_string()),
        false,
        collected
            .into_iter()
            .map(|r| RecordingMBID::from(r.mbid))
            .collect(),
        Some(
            "Automatically generated by: https://github.com/RustyNova016/listenbrainz-cli-tools"
                .to_string(),
        ),
    )
    .send(&token)
    .await
    .expect("Couldn't send playlist");
}

#[derive(Debug)]
pub struct RadioCircle {
    unlistened: bool,
    artist_blacklist: Vec<String>,
    recording_blacklist: Vec<String>,
}

impl RadioCircle {
    pub fn new(unlistened: bool) -> Self {
        Self {
            unlistened,
            ..Default::default()
        }
    }

    async fn get_random_recording_from_artist(
        &self,
        conn: &mut sqlx::SqliteConnection,
        artist: &Artist,
    ) -> Result<Option<Recording>, crate::Error> {
        println_cli(format!("Checking artist: {}", artist.name));
        let mut recordings: Vec<Recording> = artist
            .browse_or_fetch_artist_recordings(conn)
            .try_collect()
            .await?;

        recordings.shuffle(&mut thread_rng());

        for recording in recordings {
            if self.recording_blacklist.contains(&recording.mbid) {
                continue;
            }

            return Ok(Some(recording));
        }

        Ok(None)
    }

    async fn get_random_artist_from_recordings(
        &self,
        conn: &mut sqlx::SqliteConnection,
        mut recordings: Vec<&Recording>,
    ) -> Result<Option<Artist>, crate::Error> {
        recordings.shuffle(&mut thread_rng());

        for recording in recordings {
            let mut artists = recording.get_artists_or_fetch(conn).await?;

            artists.shuffle(&mut thread_rng());

            for artist in artists {
                if self.artist_blacklist.contains(&artist.mbid) {
                    continue;
                }

                return Ok(Some(artist));
            }
        }

        Ok(None)
    }

    /// Get an item of the playlist
    async fn get_random_item(
        &mut self,
        conn: &mut sqlx::SqliteConnection,
        recordings: Vec<&Recording>,
    ) -> Result<Option<Recording>, crate::Error> {
        if self.unlistened {
            recordings
                .iter()
                .for_each(|r| self.recording_blacklist.push(r.mbid.clone()));
        }

        loop {
            let artist = self
                .get_random_artist_from_recordings(conn, recordings.clone())
                .await?;

            match artist {
                Some(artist) => {
                    let recording = self.get_random_recording_from_artist(conn, &artist).await?;

                    match recording {
                        Some(recording) => {
                            self.recording_blacklist.push(recording.mbid.clone());
                            return Ok(Some(recording));
                        }
                        None => {
                            println_cli(format!("{} has not enough recordings for generation. Consider adding more recordings to Musicbrainz!", artist.name));
                            self.artist_blacklist.push(artist.mbid.clone());
                        }
                    }
                }
                None => return Ok(None),
            }
        }
    }

    pub fn into_stream<'conn, 'recordings>(
        mut self,
        conn: &'conn mut sqlx::SqliteConnection,
        recordings: Vec<&'recordings Recording>,
    ) -> impl Stream<Item = Result<Recording, crate::Error>> + use<'conn, 'recordings> {
        try_fn_stream(|emitter| async move {
            while let Some(val) = self.get_random_item(conn, recordings.clone()).await? {
                emitter.emit(val).await;
            }

            Ok(())
        })
    }
}
impl Default for RadioCircle {
    fn default() -> Self {
        Self {
            unlistened: false,
            artist_blacklist: vec![
                "125ec42a-7229-4250-afc5-e057484327fe".to_string(), // Ignore [unknown]
                "89ad4ac3-39f7-470e-963a-56509c546377".to_string(), // Ignore Verious Artist
            ],
            recording_blacklist: Vec::new(),
        }
    }
}
