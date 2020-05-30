use serde::{Deserialize, Serialize};

pub use plist::{
    from_bytes, from_file, from_reader, from_reader_xml, to_file_binary, to_file_xml,
    to_writer_binary, to_writer_xml,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebResource {
    #[serde(rename = "WebResourceData", with = "serde_bytes")]
    /// The raw binary data of the resource.
    ///
    /// This data should be interpreted based upon the values of
    /// `mime_type` and, if a text format, `text_encoding_name`.
    pub data: Vec<u8>,

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
    pub frame_name: Option<String>,

    #[serde(rename = "WebResourceMIMEType")]
    /// The MIME type of the resource.
    pub mime_type: String,

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
    /// Extended data about the server response.
    ///
    /// Usually contains a `plist` file whose contents provide further
    /// information about the HTTP response for the resource.
    pub response: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WebArchive {
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
    pub subframe_archives: Option<Vec<WebArchive>>,
}

impl WebArchive {
    pub fn print_list(&self) {
        let empty_subresources: Vec<_> = vec![];
        let empty_subframe_archives: Vec<_> = vec![];
        let subresources: &Vec<WebResource> =
            self.subresources.as_ref().unwrap_or(&empty_subresources);
        let subframe_archives: &Vec<WebArchive> = self
            .subframe_archives
            .as_ref()
            .unwrap_or(&empty_subframe_archives);

        println!(
            "WebArchive of \"{}\": {} subresource(s)",
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

#[cfg(test)]
mod tests {
    #[test]
    fn parse_crouton() {
        let bytes = include_bytes!("../fixtures/crouton.webarchive");

        let webarchive: super::WebArchive =
            super::from_bytes(bytes).expect("Could not read Crouton webarchive fixture");

        // test main resource
        assert_eq!(webarchive.main_resource.data.len(), 134);
        assert_eq!(
            webarchive.main_resource.data.get(0..10).expect("No data"),
            [60, 104, 116, 109, 108, 62, 60, 104, 101, 97]
        );
        assert_eq!(webarchive.main_resource.url, "https://crouton.net/");
        assert!(webarchive.main_resource.frame_name.is_none());

        assert_eq!(webarchive.main_resource.mime_type, "text/html");

        let text_encoding_name = webarchive
            .main_resource
            .text_encoding_name.as_ref()
            .expect("text_encoding_name not found");
        assert_eq!(text_encoding_name, "UTF-8");

        assert!(webarchive.main_resource.response.is_none());

        // test subresource png
        let subresources = webarchive.subresources.as_ref().expect("No subresources found");
        assert_eq!(subresources.len(), 1);

        let first_subresource = &subresources[0];
        assert_eq!(first_subresource.data.len(), 5182);
        assert_eq!(
            first_subresource.data.get(0..10).expect("No data"),
            [137, 80, 78, 71, 13, 10, 26, 10, 0, 0]
        );
        assert_eq!(first_subresource.url, "https://crouton.net/crouton.png");
        assert!(first_subresource.frame_name.is_none());
        assert_eq!(first_subresource.mime_type, "image/png");

        assert!(first_subresource.text_encoding_name.is_none());

        let response = first_subresource
            .response
            .as_ref()
            .expect("response not found");
        assert_eq!(response.len(), 1825);
        assert_eq!(
            response.get(0..10).expect("No data"),
            [98, 112, 108, 105, 115, 116, 48, 48, 212, 1]
        );

        // test no subframe_archives
        assert!(webarchive.subframe_archives.is_none());

        // super::to_file_binary(std::path::Path::new("./crouton.output.webarchive"), &webarchive);
    }
}
