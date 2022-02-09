use std::{io::Cursor, ops::Range};

use serde::Serialize;

use crate::error::AppErr;

use super::room::{RoomSync, RoomSyncKind};

#[derive(Debug)]
pub struct StreamHelper {
    client: reqwest::Client,
    stream_bytes: Vec<u8>,
    url: String,
    pub content_length: u64,
}

impl StreamHelper {
    const MAX_BUFFER: u64 = 1024 * 1024;

    pub fn new(content_length: u64, url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            stream_bytes: Vec::with_capacity(content_length as usize),
            url,
            content_length,
        }
    }

    pub async fn striim(&mut self, rng: Range<usize>) -> Result<&[u8], AppErr> {
        let owned_len = self.stream_bytes.len();
        if owned_len >= rng.end {
            Ok(&self.stream_bytes[rng])
        } else {
            let required = std::cmp::min(rng.end - owned_len, Self::MAX_BUFFER as usize);
            let end = owned_len + required;
            let r = self
                .client
                .get(&self.url)
                .header("Range", format!("bytes={}-{}", owned_len, end))
                .send()
                .await?;
            r.headers();
            let w = tokio::io::copy(&mut Cursor::new(r.bytes().await?), &mut self.stream_bytes)
                .await
                .map_err(AppErr::InternalError)?;
            Ok(&self.stream_bytes[rng.start..rng.start + w as usize])
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct Playing {
    pub last_room_query: String,
    pub pos: f32,
    pub is_playing: bool,
    pub is_loaded: bool,

    #[serde(skip)]
    pub stream_helper: Option<StreamHelper>,
}
impl Playing {
    pub async fn set_stream(&mut self, video_id: &str, video_query: String) -> Result<(), AppErr> {
        let aformats = yt_rs::get_stream(video_id, "en-GB", "US").await?;
        for f in aformats {
            if Some(String::from("AUDIO_QUALITY_MEDIUM")) == f.audio_quality
                && f.mime_type.contains("audio/mp4")
            {
                self.last_room_query = video_query;
                self.is_loaded = true;
                self.pos = 0f32;
                let content_length = f.content_length.parse().unwrap(); // nevermind
                self.stream_helper = Some(StreamHelper::new(content_length, f.url));
                return Ok(());
            }
        }
        Err(AppErr::ImpossibleError)
    }

    pub fn sync(&mut self, msg: &RoomSync) {
        self.pos = msg.pos;
        match msg.kind {
            RoomSyncKind::Pause => self.is_playing = false,
            RoomSyncKind::Resume => self.is_playing = true,
        }
    }
}
