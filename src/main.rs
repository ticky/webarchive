use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebResource {
    #[serde(rename = "WebResourceData", with = "serde_bytes")]
    data: Vec<u8>,
  
    #[serde(rename = "WebResourceURL")]
    url: String,

    #[serde(rename = "WebResourceFrameName", default, deserialize_with = "ruma_serde::empty_string_as_none")]
    frame_name: Option<String>,
  
    #[serde(rename = "WebResourceMIMEType", deserialize_with = "ruma_serde::empty_string_as_none")]
    mime_type: Option<String>,
  
    #[serde(rename = "WebResourceTextEncodingName", default, deserialize_with = "ruma_serde::empty_string_as_none")]
    text_encoding_name: Option<String>,

    #[serde(rename = "WebResourceResponse", default, with = "serde_bytes")]
    response: Option<Vec<u8>>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Webarchive {
    #[serde(rename = "WebMainResource")]
    main_resource: WebResource,

    #[serde(rename = "WebSubresources")]
    subresources: Option<Vec<WebResource>>,
}

fn main() {
    let webarchive: Webarchive = plist::from_file("minidisc.webarchive")
        .expect("failed to read minidisc.webarchive");

    println!("{:?}", webarchive);
}
