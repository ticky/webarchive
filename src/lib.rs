use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebResource {
    #[serde(rename = "WebResourceData", with = "serde_bytes")]
    data: Vec<u8>,

    #[serde(rename = "WebResourceURL")]
    /// The URL the resource represents.
    pub url: String,

    #[serde(
        rename = "WebResourceFrameName",
        default,
        deserialize_with = "ruma_serde::empty_string_as_none"
    )]
    /// In multi-frame pages, the name of the frame this
    /// resource is for.
    ///
    /// Corresponds to the `<frame>` element's `name` attribute:
    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Element/frame>
    frame_name: Option<String>,

    #[serde(
        rename = "WebResourceMIMEType",
        deserialize_with = "ruma_serde::empty_string_as_none"
    )]
    /// The MIME type of the resource.
    ///
    /// If omitted, the browser will attempt to
    /// infer a type automatically.
    pub mime_type: Option<String>,

    #[serde(
        rename = "WebResourceTextEncodingName",
        default,
        deserialize_with = "ruma_serde::empty_string_as_none"
    )]
    /// The text encoding used for the resource's data.
    ///
    /// May be omitted for binary data, or if the browser should
    /// attempt to automatically detect the encoding.
    pub text_encoding_name: Option<String>,

    #[serde(rename = "WebResourceResponse", default, with = "serde_bytes")]
    response: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Webarchive {
    #[serde(rename = "WebMainResource")]
    /// The main resource which the browser should display
    /// upon opening the webarchive file.
    ///
    /// May be an HTML document, or any other
    /// type the browser accepts for display.
    pub main_resource: WebResource,

    #[serde(rename = "WebSubresources")]
    /// List of subresources which the page may reference.
    /// These subresources will be loaded instead of the live
    /// version on the web if they match a requested resource.
    ///
    /// If a subresource is not provided by the webarchive
    /// file, the live version will be loaded from the network.
    pub subresources: Option<Vec<WebResource>>,

    #[serde(rename = "WebSubframeArchives")]
    /// List of archives for subframes within an archive.
    pub subframe_archives: Option<Vec<Webarchive>>,
}

impl Webarchive {
    pub fn print_list(&self) {
        let empty_subresources: Vec<_> = vec![];
        let empty_subframe_archives: Vec<_> = vec![];
        let subresources: &Vec<WebResource> =
            self.subresources.as_ref().unwrap_or(&empty_subresources);
        let subframe_archives: &Vec<Webarchive> = self
            .subframe_archives
            .as_ref()
            .unwrap_or(&empty_subframe_archives);

        println!(
            "Webarchive of \"{}\": {} subresource(s)",
            self.main_resource.url,
            subresources.len()
        );

        subresources
            .iter()
            .for_each(|subresource| println!("  - \"{}\"", subresource.url));

        subframe_archives
            .iter()
            .for_each(|subresource| subresource.print_list())
    }
}
