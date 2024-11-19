use chrono::Duration;
use futures::stream;
use futures::StreamExt;

use crate::database::get_conn;
use crate::datastructures::radio::collector::RadioCollector;
use crate::datastructures::radio::filters::cooldown::cooldown_filter;
use crate::datastructures::radio::filters::min_listens::min_listen_filter;
use crate::datastructures::radio::filters::timeouts::timeout_filter;
use crate::datastructures::radio::seeders::listens::ListenSeederBuilder;
use crate::datastructures::radio::sorters::listen_rate::listen_rate_sorter;
use crate::models::data::musicbrainz::recording::mbid::RecordingMBID;
use crate::utils::playlist::PlaylistStub;
use crate::utils::println_cli;

pub async fn listen_rate_radio(
    username: &str,
    token: &str,
    min_listens: Option<u64>,
    cooldown: u64,
    collector: RadioCollector,
) -> color_eyre::Result<()> {
    let conn = &mut *get_conn().await;

    println_cli("[Seeding] Getting listens");
    let recordings = ListenSeederBuilder::default()
        .username(username)
        .build()
        .seed(conn)
        .await
        .expect("Couldn't find seed listens");

    println_cli("[Filter] Filtering minimum listen count");
    let recordings = min_listen_filter(recordings.into_values_stream(), min_listens.unwrap_or(3));

    println_cli("[Filter] Filtering listen cooldown");
    let recordings = cooldown_filter(recordings, Duration::hours(cooldown as i64));

    println_cli("[Filter] Filtering listen timeouts");
    let recordings = timeout_filter(recordings);

    println_cli("[Sorting] Sorting listen by listen rate duration");
    let recordings = listen_rate_sorter(recordings.collect().await);

    println_cli("[Finalising] Creating radio playlist");
    let collected = collector.collect(stream::iter(recordings).map(|r| r.recording().clone())).await;

    println_cli("[Sending] Sending radio playlist to listenbrainz");
    PlaylistStub::new(
        "Radio: Listen Rate".to_string(),
        Some(username.to_string()),
        true,
        collected
            .into_iter()
            .map(|r| RecordingMBID::from(r.mbid))
            .collect(),
        Some(
            "Automatically generated by: https://github.com/RustyNova016/listenbrainz-cli-tools"
                .to_string(),
        ),
    )
    .send(token)
    .await?;

    Ok(())
}
