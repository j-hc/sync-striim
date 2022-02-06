use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub streaming_data: StreamingData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {
    pub adaptive_formats: Vec<AdaptiveFormat>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveFormat {
    pub itag: i64,
    pub url: String,
    pub mime_type: String,
    pub bitrate: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    // pub init_range: InitRange,
    // pub index_range: IndexRange,
    pub last_modified: String,
    pub content_length: String,
    pub quality: String,
    pub fps: Option<i64>,
    pub quality_label: Option<String>,
    pub projection_type: String,
    pub average_bitrate: Option<i64>,
    pub approx_duration_ms: String,
    // pub color_info: Option<ColorInfo>,
    pub high_replication: Option<bool>,
    pub audio_quality: Option<String>,
    pub audio_sample_rate: Option<String>,
    pub audio_channels: Option<i64>,
}
