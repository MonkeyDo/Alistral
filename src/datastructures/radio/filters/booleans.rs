use core::future::ready;

use futures::StreamExt;

use crate::datastructures::entity_with_listens::recording_with_listens::RecordingWithListens;

pub fn and_filter(
    radio: impl StreamExt<Item = RecordingWithListens>,
    other: Vec<RecordingWithListens>,
) -> impl StreamExt<Item = RecordingWithListens> {
    radio.filter(move |track| {
        ready(
            other
                .iter()
                .any(|other_track| track.recording().mbid == other_track.recording().mbid),
        )
    })
}
