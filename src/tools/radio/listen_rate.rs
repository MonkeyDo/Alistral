use chrono::Duration;
use futures::stream;
use futures::StreamExt;
use interzic::models::playlist_stub::PlaylistStub;

use crate::datastructures::radio::collector::RadioCollector;
use crate::datastructures::radio::filters::cooldown::cooldown_filter;
use crate::datastructures::radio::filters::min_listens::min_listen_filter;
use crate::datastructures::radio::filters::timeouts::timeout_filter;
use crate::datastructures::radio::seeders::listens::ListenSeeder;
use crate::datastructures::radio::sorters::listen_rate::listen_rate_sorter;
use crate::models::cli::radio::RadioExportTarget;
use crate::models::data_storage::DataStorage;
use crate::tools::radio::convert_recordings;
use crate::utils::data_file::DataFile as _;
use crate::utils::println_cli;

pub async fn listen_rate_radio(
    conn: &mut sqlx::SqliteConnection,
    seeder: ListenSeeder,
    token: &str,
    min_listens: Option<u64>,
    cooldown: u64,
    collector: RadioCollector,
    target: RadioExportTarget,
) -> color_eyre::Result<()> {
    let username = seeder.username().clone();

    println_cli("[Seeding] Getting listens");
    let recordings = seeder.seed(conn).await.expect("Couldn't find seed listens");

    println_cli("[Filter] Filtering minimum listen count");
    let recordings = min_listen_filter(recordings.into_stream(), min_listens.unwrap_or(3));

    println_cli("[Filter] Filtering listen cooldown");
    let recordings = cooldown_filter(recordings, Duration::hours(cooldown as i64));

    println_cli("[Filter] Filtering listen timeouts");
    let recordings = timeout_filter(recordings);

    println_cli("[Sorting] Sorting listen by listen rate duration");
    let recordings = listen_rate_sorter(recordings.collect().await);

    println_cli("[Finalising] Creating radio playlist");
    let collected = collector
        .collect(stream::iter(recordings).map(|r| r.recording().clone()))
        .await;

    println_cli("[Sending] Sending radio playlist to listenbrainz");

    let counter = DataStorage::load().expect("Couldn't load data storage");
    let playlist = PlaylistStub {
        title: format!(
            "Radio: Listen Rate #{}",
            counter.write().unwrap().incr_playlist_count()
        ),
        description: "Automatically generated by: https://github.com/RustyNova016/Alistral"
            .to_string(),
        recordings: convert_recordings(conn, collected)
            .await
            .expect("Couldn't convert recordings for playlist"),
    };

    target
        .export(playlist, Some(username), Some(token))
        .await
        .expect("Couldn't send the playlist");

    Ok(())
}
