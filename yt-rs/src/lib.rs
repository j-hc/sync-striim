mod search_models;
mod stream_models;

use reqwest::header::HeaderValue;
use serde_json::json;

const SEARCH_URL: &str =
    "https://www.youtube.com/youtubei/v1/search?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const STREAM_URL: &str = "https://youtubei.googleapis.com/youtubei/v1/player?key=AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w";
const WEB_USERAGENT: &str = "Mozilla/5.0 (Windows NT 10.0; rv:78.0) Gecko/20100101 Firefox/78.0";
const MOBILE_USERAGENT: &str = "com.google.android.youtube/16.29.38Linux; U; Android 11; US) gzip";
const X_YOUTUBE_CLIENT_VERSION: &str = "2.20210728.00.00";
const MOBILE_CLIENT_VERSION: &str = "16.29.38";

pub struct VideoRS {
    inner: search_models::Content2,
}
impl VideoRS {
    pub fn video_id(&self) -> &str {
        &self.inner.video_renderer.as_ref().unwrap().video_id
    }

    pub fn title(&self) -> &str {
        &self.inner.video_renderer.as_ref().unwrap().title.runs[0].text
    }
}

macro_rules! headermap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::reqwest::header::HeaderMap::new();
         $( map.insert($key, HeaderValue::from_static($val)); )*
         map
    }}
}

pub async fn search(query: &str, hl: &str, gl: &str) -> reqwest::Result<Vec<VideoRS>> {
    let headers = headermap![
        "Host" => "www.youtube.com",
        "user-agent" => WEB_USERAGENT,
        "accept-language" => "en-GB, en;q=0.9",
        "origin" => "https://www.youtube.com",
        "x-youtube-client-name" => "1",
        "referer" => "https://www.youtube.com",
        "x-youtube-client-version" => X_YOUTUBE_CLIENT_VERSION,
        "content-type" => "application/json"
    ];

    let body = json!({
      "query": query,
      "context": {
        "client": {
          "hl": hl,
          "gl": gl,
          "clientName": "WEB",
          "clientVersion": X_YOUTUBE_CLIENT_VERSION
        },
        "user": {
          "lockedSafetyMode": false
        }
      }
    });

    let r = reqwest::Client::new()
        .post(SEARCH_URL)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    let mut v: search_models::Root = r.json().await?;

    Ok(v.contents
        .two_column_search_results_renderer
        .primary_contents
        .section_list_renderer
        .contents[0]
        .item_section_renderer
        .take()
        .unwrap()
        .contents
        .into_iter()
        .map(|c| VideoRS { inner: c })
        .collect())
}

pub async fn get_stream(
    video_id: &str,
    hl: &str,
    gl: &str,
) -> reqwest::Result<Vec<stream_models::AdaptiveFormat>> {
    let headers = headermap![
        "Host" => "youtubei.googleapis.com",
        "user-agent" => MOBILE_USERAGENT,
        "accept-language" => "en-GB, en;q=0.9",
        "x-goog-api-format-version" => "2",
        "content-type" => "application/json"
    ];

    let body = json!({
      "context": {
        "client": {
          "hl": hl,
          "gl": gl,
          "clientName": "ANDROID",
          "clientVersion": MOBILE_CLIENT_VERSION
        },
        "user": {
          "lockedSafetyMode": false
        }
      },
      "videoId": video_id
    });

    let r = reqwest::Client::new()
        .post(STREAM_URL)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    let v: stream_models::Root = r.json().await?;

    Ok(v.streaming_data.adaptive_formats)
}

#[cfg(test)]
mod tests {
    use reqwest::ResponseBuilderExt;

    use crate::*;

    #[tokio::test]
    async fn test_search() {
        let s = search("heyyo", "en-GB", "US").await;
        assert!(s.is_ok());
        let v = s.unwrap();
        assert!(!v.is_empty());
    }

    #[tokio::test]
    async fn test_stream() {
        let s = get_stream("jKhP750VdXw", "en-GB", "US").await;
        assert!(s.is_ok());
        let v = s.unwrap();
        assert!(!v.is_empty());
        for f in v {
            if Some(String::from("AUDIO_QUALITY_MEDIUM")) == f.audio_quality
                && f.mime_type.contains("audio/mp4")
            {
                println!("{}", f.url)
            }
        }
    }
}
