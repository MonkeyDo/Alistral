use std::sync::Arc;

use extend::ext;

use crate::models::data::listenbrainz::listen::listen_spe::ListenSpe;
use crate::models::data::listenbrainz::listen::listen_spe::Mapped;
use crate::models::data::listenbrainz::listen::listen_unspe::ListenMappingState;

#[ext]
pub impl Vec<ListenMappingState> {
    fn into_mapped_collection(self) -> Vec<Arc<ListenSpe<Mapped>>> {
        self.into_iter().filter_map(|listen| listen.as_mapped().cloned()).collect()
    }
}