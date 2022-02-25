# webarchive

[![Rust](https://github.com/ticky/webarchive/actions/workflows/rust.yml/badge.svg)](https://github.com/ticky/webarchive/actions/workflows/rust.yml)

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
Web Archive files. I also intend to write a command line utility based on
this API which allows bi-directional conversion between common formats and
Web Archives.

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

webarchive::to_writer_xml(&mut buf, &archive)
    .expect("should write xml");

assert_eq!(
    String::from_utf8(buf).expect("should contain utf-8"),
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
