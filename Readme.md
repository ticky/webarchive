# webarchive

[![crates.io](https://img.shields.io/crates/v/webarchive.svg)](https://crates.io/crates/webarchive) [![docs.rs](https://img.shields.io/docsrs/webarchive)](https://docs.rs/webarchive/) [![Rust](https://github.com/ticky/webarchive/actions/workflows/rust.yml/badge.svg)](https://github.com/ticky/webarchive/actions/workflows/rust.yml)

Rust utilities for working with Apple's Web Archive file format,
as produced by Safari 2 or later on macOS, Safari 4 or later on Windows,
or Safari 13 or later on iOS and iPadOS.

## Why Web Archive?

Web Archive files have been around since 2005, and are a way to save an
entire web page, and all associated resources involved in displaying it,
as a single file which can be saved to disk, reviewed or shared regardless
of changes, removals or the state of the server which originally served it.

While not well supported outside of Apple platforms, and not supported by
iOS until iOS 13 in 2019, the Web Archive is one of few formats designed
for a user to simply open a page and expect it to work as the original did.
One which came closest is [MHTML](https://en.wikipedia.org/wiki/MHTML),
supported in older versions of Microsoft's Internet Explorer and with a
similar approach to Web Archive, representing a web page in its entirety.

Alternatives aimed at professional or semi-professional archives work, such
as [WARC](https://en.wikipedia.org/wiki/Web_ARChive) instead represent an
entire browsing session and associated subresources, but require
specialised software to view, and do not have the concept of a "main" page
or resource. Web Archives, by contrast, open in a normal web browser and
do not require the user to know which URL to select.

## Okay, so what's the goal?

I aim for this to be an ergonomic API for reading, creating, and converting
Web Archive files, and to expand the included command line utility to allow
bi-directional conversion between common formats and Web Archives.

## Usage

### Command-line usage

A command-line utility is provided, which can be installed by running:

```shell
cargo install webarchive
```

This utility can extract or inspect the contents of webarchive files.

List the contents with `inspect`:

```shell
$ webarchive inspect fixtures/psxdatacenter.webarchive
WebArchive of "http://psxdatacenter.com/ntsc-j_list.html": 0 subresource(s)
WebArchive of "http://psxdatacenter.com/banner.html": 2 subresource(s)
  - "http://psxdatacenter.com/images/texgrey.jpg"
  - "http://psxdatacenter.com/images/logo.jpg"
WebArchive of "http://psxdatacenter.com/nav.html": 16 subresource(s)
  - "http://psxdatacenter.com/images/texgrey.jpg"
  - "http://psxdatacenter.com/buttons/news1.gif"
  - "http://psxdatacenter.com/buttons/inf1.gif"
  - "http://psxdatacenter.com/buttons/emul1.gif"
...
```

Or extract them to disk with `extract`:

```shell
$ webarchive extract fixtures/psxdatacenter.webarchive
Saving main resource...
Writing file "fixtures/psxdatacenter.com/ntsc-j_list.html"...
Saving subframe archives...
Saving main resource...
Writing file "fixtures/psxdatacenter.com/banner.html"...
Saving subresources...
Writing file "fixtures/psxdatacenter.com/images/texgrey.jpg"...
Writing file "fixtures/psxdatacenter.com/images/logo.jpg"...
...
```

### Reading a webarchive

```rust
use webarchive::WebArchive;

let archive: WebArchive = webarchive::from_file("fixtures/psxdatacenter.webarchive")?;

/// main_resource is the resource which is opened by default
assert_eq!(
    archive.main_resource.url,
    "http://psxdatacenter.com/ntsc-j_list.html"
);
assert_eq!(archive.main_resource.mime_type, "text/html");
assert_eq!(
    archive.main_resource.text_encoding_name,
    Some("UTF-8".to_string())
);
assert_eq!(archive.main_resource.data.len(), 2171);
assert!(archive.subresources.is_none());

/// subframe_archives contains additional WebArchives for frames
assert!(archive.subframe_archives.is_some());
let subframe_archives = archive.subframe_archives.unwrap();
assert_eq!(subframe_archives.len(), 4);

assert_eq!(
    subframe_archives[0].main_resource.url,
    "http://psxdatacenter.com/banner.html"
);
assert_eq!(subframe_archives[0].main_resource.mime_type, "text/html");
assert_eq!(
    subframe_archives[0].main_resource.text_encoding_name,
    Some("UTF-8".to_string())
);
assert_eq!(subframe_archives[0].main_resource.data.len(), 782);

/// subresources are the files referenced by a given frame
assert!(subframe_archives[0].subresources.is_some());
let subresources = subframe_archives[0].subresources.as_ref().unwrap();
assert_eq!(subresources.len(), 2);

assert_eq!(
    subresources[0].url,
    "http://psxdatacenter.com/images/texgrey.jpg"
);
assert_eq!(subresources[0].mime_type, "image/jpeg");
assert!(subresources[0].text_encoding_name.is_none());
assert_eq!(subresources[0].data.len(), 107128);
```

### Creating a webarchive

```rust
use webarchive::{WebArchive, WebResource};

let resource = WebResource {
    url: "about:hello".to_string(),
    data: "hello world".as_bytes().to_vec(),
    mime_type: "text/plain".to_string(),
    text_encoding_name: Some("utf-8".to_string()),
    frame_name: None,
    response: None,
};

let archive = WebArchive {
    main_resource: resource,
    subresources: None,
    subframe_archives: None,
};

let mut buf: Vec<u8> = Vec::new();

webarchive::to_writer_xml(&mut buf, &archive)?;

assert_eq!(
    String::from_utf8(buf)?,
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>WebMainResource</key>
	<dict>
		<key>WebResourceData</key>
		<data>
		aGVsbG8gd29ybGQ=
		</data>
		<key>WebResourceURL</key>
		<string>about:hello</string>
		<key>WebResourceMIMEType</key>
		<string>text/plain</string>
		<key>WebResourceTextEncodingName</key>
		<string>utf-8</string>
	</dict>
</dict>
</plist>"#
);
```