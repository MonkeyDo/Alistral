use musicbrainz_db_lite::models::musicbrainz::{main_entities::MainEntity, recording::Recording};

use crate::models::clippy::{MbClippyLint, MbClippyLintLink};

pub struct MissingWorkLint {
    recording: Recording,
}

impl MbClippyLint for MissingWorkLint {
    fn get_name() -> &'static str {
        "missing_recording_work"
    }

    async fn check(
        conn: &mut sqlx::SqliteConnection,
        entity: &MainEntity,
    ) -> Result<Option<Self>, crate::Error> {
        let MainEntity::Recording(recording) = entity else {
            return Ok(None);
        };

        let work = recording.get_works_or_fetch(conn).await?;

        if !work.is_empty() {
            return Ok(None);
        }

        let missing_work_lint = Self {
            recording: recording.clone(),
        };

        Ok(Some(missing_work_lint))
    }

    async fn get_body(
        &self,
        conn: &mut sqlx::SqliteConnection,
    ) -> Result<impl std::fmt::Display, crate::Error> {
        Ok(format!("Recording \"{}\" has no associated works
-> Recordings should have works associated to them. Please check if a work exists for a recording and add it / create it"
, self.recording.format_with_credits(conn).await?))
    }

    async fn get_links(
        &self,
        conn: &mut sqlx::SqliteConnection,
    ) -> Result<Vec<MbClippyLintLink>, crate::Error> {
        let mut out = Vec::new();
        let releases = self.recording.get_releases_or_fetch(conn).await?;

        out.push(MbClippyLintLink {
            name: "Recording".to_string(),
            url: format!("https://musicbrainz.org/recording/{}", self.recording.mbid),
        });

        if let Some(release) = releases.first() {
            out.push(MbClippyLintLink {
                name: "Release relations".to_string(),
                url: format!(
                    "https://musicbrainz.org/release/{}/edit-relationships",
                    release.mbid
                ),
            });
        }

        Ok(out)
    }

    async fn get_hints(
        &self,
        conn: &mut sqlx::SqliteConnection,
    ) -> Result<Vec<crate::models::clippy::MbClippyLintHint>, crate::Error> {
        // TODO: Remix hint
        Ok(Vec::new())
    }
}
