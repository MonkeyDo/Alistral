use core::pin::Pin;

use alistral_core::datastructures::entity_with_listens::recording::RecordingWithListens;
use chrono::Duration;
use futures::{stream, StreamExt};
use interzic::models::messy_recording::MessyRecording;
use interzic::models::playlist_stub::PlaylistStub;
use interzic::models::services::youtube::Youtube;
use interzic::Client;
use listenbrainz::raw::jspf::Playlist;
use musicbrainz_db_lite::client::MusicBrainzClient;

use crate::api::youtube::SYMPHONYZ_DB;
use crate::api::youtube::TOKENCACHE;
use crate::api::youtube::YT_SECRET_FILE;
use crate::datastructures::radio::collector::RadioCollector;
use crate::datastructures::radio::filters::cooldown::cooldown_filter;
use crate::datastructures::radio::filters::min_listens::min_listen_filter;
use crate::datastructures::radio::filters::timeouts::timeout_filter;
use crate::datastructures::radio::seeders::listens::ListenSeeder;
use crate::datastructures::radio::sorters::overdue::overdue_factor_sorter;
use crate::datastructures::radio::sorters::overdue::overdue_factor_sorter_cumulative;
use crate::datastructures::radio::sorters::overdue::overdue_sorter;
use crate::models::data_storage::DataStorage;
use crate::utils::data_file::DataFile;
use crate::utils::println_cli;

//TODO: Refactor Radios params into structs
#[expect(clippy::too_many_arguments)]
pub async fn overdue_radio(
    conn: &mut sqlx::SqliteConnection,
    seeder: ListenSeeder,
    token: &str,
    min_listens: Option<u64>,
    cooldown: u64,
    overdue_factor: bool,
    collector: RadioCollector,
    at_listening_time: bool,
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

    let recordings = if !overdue_factor {
        println_cli("[Sorting] Sorting listen by overdue duration");
        Box::pin(stream::iter(overdue_sorter(recordings.collect().await)))
            as Pin<Box<dyn futures::Stream<Item = RecordingWithListens>>>
    } else if !at_listening_time {
        println_cli("[Sorting] Sorting listen by overdue factor");
        Box::pin(stream::iter(overdue_factor_sorter(
            recordings.collect().await,
        ))) as Pin<Box<dyn futures::Stream<Item = RecordingWithListens>>>
    } else {
        println_cli("[Sorting] Sorting listen by overdue factor at listen time");
        Box::pin(overdue_factor_sorter_cumulative(recordings.collect().await))
    };

    println_cli("[Finalising] Creating radio playlist");
    let collected = collector
        .collect(recordings.map(|r| r.recording().clone()))
        .await;

    println_cli("[Sending] Sending radio playlist to listenbrainz");
    let counter = DataStorage::load().expect("Couldn't load data storage");
    // PlaylistStub::new(
    //     format!(
    //         "Radio: Overdue listens #{}",
    //         counter.write().unwrap().incr_playlist_count()
    //     ),
    //     Some(username.to_string()),
    //     true,
    //     collected.into_iter().map(|r| r.mbid).collect(),
    //     Some("Automatically generated by: https://github.com/RustyNova016/Alistral".to_string()),
    // )
    // .send(token)
    // .await?;

        let mut client = Client::new_builder();
    client.set_musicbrainz_client(MusicBrainzClient::default());
    client.create_database_if_missing(&SYMPHONYZ_DB).unwrap();
    client
        .read_database(&SYMPHONYZ_DB.to_string_lossy())
        .unwrap();
    //client.read_database(&SYMPHONYZ_DB).unwrap();
    client.migrate_database().await.unwrap();
    let mut client = client.build().unwrap();
    client
        .set_youtube_client(&YT_SECRET_FILE, &TOKENCACHE)
        .await
        .unwrap();

    let mut messy = Vec::new();
    for recording in collected {
        let rec = MessyRecording::from_db_recording(conn, recording).await?;
        let rec = rec.upsert(&client.database_client).await?;
        messy.push(rec);
    }

    let playlist = PlaylistStub {
        title: format!(
            "Radio: Overdue listens #{}",
            counter.write().unwrap().incr_playlist_count()
        ),
        description: "Automatically generated by: https://github.com/RustyNova016/Alistral"
            .to_string(),
        recordings: messy,
    };



    Youtube::create_playlist(&client, playlist).await?;

    Ok(())
}

// #[tokio::test]
// #[serial_test::serial]
// async fn overdue_by() {
//     use crate::datastructures::radio::collector::RadioCollectorBuilder;
//     overdue_radio(
//         "RustyNova",
//         "t",
//         None,
//         0,
//         false,
//         RadioCollectorBuilder::default()
//             .count_default()
//             .duration_default()
//             .build(),
//             true
//     )
//     .await
//     .unwrap();
// }
