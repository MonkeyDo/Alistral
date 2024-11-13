use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use derive_getters::Getters;
use musicbrainz_db_lite::models::listenbrainz::listen::Listen;
use musicbrainz_db_lite::models::musicbrainz::recording::Recording;
use musicbrainz_db_lite::models::musicbrainz::user::User;
use musicbrainz_db_lite::RowId;
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::database::listenbrainz::prefetching::prefetch_recordings_of_listens;
use crate::datastructures::listen_collection::traits::ListenCollectionLike;
use crate::datastructures::listen_collection::ListenCollection;

use super::impl_entity_with_listens;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct RecordingWithListens {
    recording: Recording,
    listens: ListenCollection,
}

impl RecordingWithListens {
    pub fn new(recording: Recording, listens: ListenCollection) -> Self {
        Self { listens, recording }
    }

    pub async fn from_listencollection(
        conn: &mut sqlx::SqliteConnection,
        listens: ListenCollection,
    ) -> Result<HashMap<i64, Self>, crate::Error> {
        // If empty, early return
        if listens.is_empty() {
            return Ok(Default::default());
        }

        // Prefetch the missing data
        let user_name = listens
            .first()
            .expect("At least one listen should be there")
            .user
            .clone();

        let user = User::find_by_name(conn, &user_name)
            .await?
            .ok_or(crate::Error::MissingUserError(user_name.clone()))?;

        prefetch_recordings_of_listens(conn, user.id, &listens.data).await?;

        // Get all the data from the DB
        let joins = Listen::get_recordings_as_batch(conn, user.id, listens.data).await?;

        // Convert into structs
        let mut out = HashMap::new();

        for (_, (listen, recordings)) in joins {
            for recording in recordings {
                out.entry(recording.get_row_id())
                    .or_insert_with(|| Self {
                        recording,
                        listens: ListenCollection::default(),
                    })
                    .push(listen.clone());
            }
        }

        Ok(out)
    }

    pub fn push(&mut self, listen: Listen) {
        self.listens.push(listen);
    }

    pub fn first_listen_date(&self) -> Option<DateTime<Utc>> {
        self.listens
            .get_oldest_listen()
            .map(|listen| listen.listened_at_as_datetime())
    }

    pub fn last_listen_date(&self) -> Option<DateTime<Utc>> {
        self.listens
            .get_latest_listen()
            .map(|listen| listen.listened_at_as_datetime())
    }

    /// Return the amount of time this recording having known about
    pub fn known_for(&self) -> Option<Duration> {
        self.first_listen_date()
            .map(|discovery| Utc::now() - discovery)
    }

    pub fn average_duration_between_listens(&self) -> Duration {
        // If the recording haven't been listened to, then the average time is zero
        if self.len() < 2 {
            return Duration::zero();
        }

        let duration_between_first_and_last = self
            .last_listen_date()
            .expect("There's at least two listens")
            - self
                .first_listen_date()
                .expect("There's at least two listens");

        duration_between_first_and_last
            .checked_div(self.len() as i32)
            .unwrap_or_else(Duration::zero)
    }

    pub fn estimated_date_of_next_listen(&self) -> Option<DateTime<Utc>> {
        self.last_listen_date()
            .map(|listen_date| listen_date + self.average_duration_between_listens())
    }

    pub fn overdue_by(&self) -> Duration {
        self.estimated_date_of_next_listen()
            .map(|next_listen| Utc::now() - next_listen)
            .unwrap_or_else(Duration::zero)
    }

    pub fn overdue_factor(&self) -> Decimal {
        Decimal::from_i64(self.overdue_by().num_seconds()).unwrap()
            / Decimal::from_i64(self.average_duration_between_listens().num_seconds()).unwrap()
    }

    pub fn is_listened(&self) -> bool {
        !self.listens.data.is_empty()
    }

    /*     pub async fn underated_score_single(&self) -> color_eyre::Result<Decimal> {
        Ok(self
            .listens()
            .get_underrated_recordings()
            .await?
            .first()
            .expect("Recording should have a score")
            .0)
    } */
}

impl_entity_with_listens!(RecordingWithListens);

impl ListenCollectionLike for RecordingWithListens {
    fn iter_listens(&self) -> impl Iterator<Item = &Listen> {
        self.listens.iter_listens()
    }
}
