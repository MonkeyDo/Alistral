use std::fmt::Write as _;

use color_eyre::owo_colors::OwoColorize;
use extend::ext;
use musicbrainz_db_lite::models::musicbrainz::artist::Artist;
use musicbrainz_db_lite::models::musicbrainz::artist_credit::ArtistCredits;
use musicbrainz_db_lite::models::musicbrainz::main_entities::MainEntity;
use musicbrainz_db_lite::models::musicbrainz::recording::Recording;
use musicbrainz_db_lite::models::musicbrainz::release::Release;

use super::hyperlink_rename;

#[ext]
pub impl MainEntity {
    async fn pretty_format(
        &self,
        conn: &mut sqlx::SqliteConnection,
    ) -> Result<String, crate::Error> {
        let out = match self {
            MainEntity::Artist(val) => val.pretty_format().await?,
            MainEntity::Label(val) => format!("{}", val.name),
            MainEntity::Recording(val) => val.pretty_format_with_credits(conn).await?,
            MainEntity::Release(val) => val.pretty_format_with_credits(conn).await?,
            MainEntity::Work(val) => format!("{}", val.title),
        };

        Ok(out)
    }
}

fn format_disambiguation(title: &str, disambiguation: &Option<String>) -> String {
    let dis = match disambiguation {
        None => "",
        Some(val) => {
            if !val.is_empty() {
                &format!(" ({})",&val).truecolor(200, 200, 200).to_string()
            } else {
                ""
            }
        }
    };

    format!("{title}{dis}")
}

#[ext]
pub impl Artist {
    async fn pretty_format(&self) -> Result<String, crate::Error> {
        Ok(hyperlink_rename(
            &self.name.truecolor(20, 163, 249),
            &format!("https://listenbrainz.org/artist/{}", &self.mbid),
        ))
    }
}

#[ext]
pub impl ArtistCredits {
    async fn pretty_format(&self) -> Result<String, crate::Error> {
        let mut out = String::new();

        for credit in &self.1 {
            write!(
                out,
                "{}{}",
                hyperlink_rename(
                    &credit.name.truecolor(20, 163, 249),
                    &format!("https://listenbrainz.org/artist/{}", &credit.artist_gid)
                ),
                credit.join_phrase
            )
            .expect("Display format is infaillible");
        }
        Ok(out)
    }
}


#[ext]
pub impl Recording {
    async fn pretty_format(&self) -> Result<String, crate::Error> {
        Ok(hyperlink_rename(
            &format_disambiguation(&self.title.truecolor(0, 214, 114).to_string(), &self.disambiguation),
            &format!("https://listenbrainz.org/artist/{}", &self.mbid),
        ))
    }

    async fn pretty_format_with_credits(&self, conn: &mut sqlx::SqliteConnection) -> Result<String, crate::Error> {
        Ok(format!("{} by {}", self.pretty_format().await?, self.get_artist_credits_or_fetch(conn).await?.pretty_format().await?))
    }
}

#[ext]
pub impl Release {
    async fn pretty_format(&self) -> Result<String, crate::Error> {
        Ok(hyperlink_rename(
            &format_disambiguation(&self.title.truecolor(242, 244, 123).to_string(), &self.disambiguation),
            &format!("https://listenbrainz.org/release/{}", &self.mbid),
        ))
    }

    async fn pretty_format_with_credits(&self, conn: &mut sqlx::SqliteConnection) -> Result<String, crate::Error> {
        Ok(format!("{} by {}", self.pretty_format().await?, self.get_artist_credits_or_fetch(conn).await?.pretty_format().await?))
    }
}

