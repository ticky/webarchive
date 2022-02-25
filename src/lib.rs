//! Utilities for working with Apple's Web Archive file format,
//! as produced by Safari 2 or later on macOS, Safari 4 or later on Windows,
//! or Safari 13 or later on iOS and iPadOS.
//!
//! ## Why Web Archive?
//!
//! Web Archive files have been around since 2005, and are a way to save an
//! entire web page, and all associated resources involved in displaying it,
//! as a single file which can be saved to disk, reviewed or shared regardless
//! of changes, removals or the state of the server which originally served it.
//!
//! While not well supported outside of Apple platforms, and not supported by
//! iOS until iOS 13 in 2019, the Web Archive is one of few formats designed
//! for a user to simply open a page and expect it to work as the original did.
//! One which came closest is [MHTML](https://en.wikipedia.org/wiki/MHTML),
//! supported in older versions of Microsoft's Internet Explorer and with a
//! similar approach to Web Archive, representing a web page in its entirety.
//!
//! Alternatives aimed at professional or semi-professional archives work, such
//! as [WARC](https://en.wikipedia.org/wiki/Web_ARChive) instead represent an
//! entire browsing session and associated subresources, but require
//! specialised software to view, and do not have the concept of a "main" page
//! or resource. Web Archives, by contrast, open in a normal web browser and
//! do not require the user to know which URL to select.
//!
//! ## Okay, so what's the goal?
//!
//! I aim for this to be an ergonomic API for reading, creating, and converting
//! Web Archive files, and to expand the included command line utility to allow
//! bi-directional conversion between common formats and Web Archives.
//!
//! ### Reading a webarchive
//!
//! ```rust
//! # use anyhow::Result;
//! use webarchive::WebArchive;
//!
//! # fn main() -> Result<()> {
//! let archive: WebArchive = webarchive::from_file("fixtures/psxdatacenter.webarchive")?;
//!
//! /// main_resource is the resource which is opened by default
//! assert_eq!(
//!     archive.main_resource.url,
//!     "http://psxdatacenter.com/ntsc-j_list.html"
//! );
//! assert_eq!(archive.main_resource.mime_type, "text/html");
//! assert_eq!(
//!     archive.main_resource.text_encoding_name,
//!     Some("UTF-8".to_string())
//! );
//! assert_eq!(archive.main_resource.data.len(), 2171);
//! assert!(archive.subresources.is_none());
//!
//! /// subframe_archives contains additional WebArchives for frames
//! assert!(archive.subframe_archives.is_some());
//! let subframe_archives = archive.subframe_archives.unwrap();
//! assert_eq!(subframe_archives.len(), 4);
//!
//! assert_eq!(
//!     subframe_archives[0].main_resource.url,
//!     "http://psxdatacenter.com/banner.html"
//! );
//! assert_eq!(subframe_archives[0].main_resource.mime_type, "text/html");
//! assert_eq!(
//!     subframe_archives[0].main_resource.text_encoding_name,
//!     Some("UTF-8".to_string())
//! );
//! assert_eq!(subframe_archives[0].main_resource.data.len(), 782);
//!
//! /// subresources are the files referenced by a given frame
//! assert!(subframe_archives[0].subresources.is_some());
//! let subresources = subframe_archives[0].subresources.as_ref().unwrap();
//! assert_eq!(subresources.len(), 2);
//!
//! assert_eq!(
//!     subresources[0].url,
//!     "http://psxdatacenter.com/images/texgrey.jpg"
//! );
//! assert_eq!(subresources[0].mime_type, "image/jpeg");
//! assert!(subresources[0].text_encoding_name.is_none());
//! assert_eq!(subresources[0].data.len(), 107128);
//! # Ok(())
//! # }
//! ```
//!
//! ### Creating a webarchive
//!
//! ```rust
//! # use anyhow::Result;
//! use webarchive::{WebArchive, WebResource};
//!
//! # fn main() -> Result<()> {
//! let resource = WebResource {
//!     url: "about:hello".to_string(),
//!     data: "hello world".as_bytes().to_vec(),
//!     mime_type: "text/plain".to_string(),
//!     text_encoding_name: Some("utf-8".to_string()),
//!     frame_name: None,
//!     response: None,
//! };
//!
//! let archive = WebArchive {
//!     main_resource: resource,
//!     subresources: None,
//!     subframe_archives: None,
//! };
//!
//! let mut buf: Vec<u8> = Vec::new();
//!
//! webarchive::to_writer_xml(&mut buf, &archive)?;
//!
//! assert_eq!(
//!     String::from_utf8(buf)?,
//!     r#"<?xml version="1.0" encoding="UTF-8"?>
//! <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
//! <plist version="1.0">
//! <dict>
//! 	<key>WebMainResource</key>
//! 	<dict>
//! 		<key>WebResourceData</key>
//! 		<data>
//! 		aGVsbG8gd29ybGQ=
//! 		</data>
//! 		<key>WebResourceURL</key>
//! 		<string>about:hello</string>
//! 		<key>WebResourceMIMEType</key>
//! 		<string>text/plain</string>
//! 		<key>WebResourceTextEncodingName</key>
//! 		<string>utf-8</string>
//! 	</dict>
//! </dict>
//! </plist>"#
//! );
//! # Ok(())
//! # }
//! ```

#![allow(clippy::tabs_in_doc_comments)]

use serde::{Deserialize, Serialize};

pub use plist::{
    from_bytes, from_file, from_reader, from_reader_xml, to_file_binary, to_file_xml,
    to_writer_binary, to_writer_xml,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Represents an individual web resource which would be requested
/// as part of displaying the page represented by the Web Archive file.
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
/// Represents an entire Web Archive file.
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
    ///
    /// In Safari, these subresources are loaded in place of the
    /// live version from the web if they match a requested resource.
    /// If a requested subresource is not provided by the webarchive
    /// file, the live version is instead loaded from the network.
    pub subresources: Option<Vec<WebResource>>,

    #[serde(rename = "WebSubframeArchives")]
    /// List of archives for subframes within an archive.
    pub subframe_archives: Option<Vec<WebArchive>>,
}

impl WebArchive {
    /// Print a list of all contained resources and subframe archives
    pub fn print_list(&self) {
        let subresource_count = match &self.subresources {
            Some(subresources) => subresources.len(),
            None => 0,
        };

        let subframe_archive_count = match &self.subframe_archives {
            Some(subframe_archives) => subframe_archives.len(),
            None => 0,
        };

        println!(
            "WebArchive of \"{}\" ({:?}, {} bytes): {} subresource{}, {} subframe archive{} totalling {} bytes",
            self.main_resource.url,
            self.main_resource.mime_type,
            self.main_resource.data.len(),
            subresource_count,
            if subresource_count == 1 { "" } else { "s" },
            subframe_archive_count,
            if subframe_archive_count == 1 { "" } else { "s" },
            self.total_size(),
        );

        if let Some(subresources) = &self.subresources {
            subresources.iter().for_each(|subresource| {
                println!(
                    "  - \"{}\" ({:?}, {} bytes)",
                    subresource.url,
                    subresource.mime_type,
                    subresource.data.len()
                )
            });
        }

        if let Some(webarchives) = &self.subframe_archives {
            webarchives
                .iter()
                .for_each(|webarchive| webarchive.print_list());
        }
    }

    /// Get the total size of all contained resources in bytes.
    ///
    /// Does not include metadata or extra response information.
    pub fn total_size(&self) -> usize {
        let subresource_size = match &self.subresources {
            Some(subresources) => subresources
                .iter()
                .map(|subresource| subresource.data.len())
                .sum(),
            None => 0,
        };

        let subframe_archive_size = match &self.subframe_archives {
            Some(webarchives) => webarchives
                .iter()
                .map(|webarchive| webarchive.total_size())
                .sum(),
            None => 0,
        };

        self.main_resource.data.len() + subresource_size + subframe_archive_size
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
            .text_encoding_name
            .as_ref()
            .expect("text_encoding_name not found");
        assert_eq!(text_encoding_name, "UTF-8");

        assert!(webarchive.main_resource.response.is_none());

        // test subresource png
        let subresources = webarchive
            .subresources
            .as_ref()
            .expect("No subresources found");
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
