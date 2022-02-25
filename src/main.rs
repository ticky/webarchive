use anyhow::Result;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use webarchive::{WebArchive, WebResource};

#[derive(StructOpt, Debug)]
#[structopt()]
struct Options {
    #[structopt(parse(from_os_str))]
    /// File or folder to convert or inspect
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    /// File or folder name to output to if converting
    output: Option<PathBuf>,
}

fn save(resource: WebResource, inside: &Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut url: String = resource
        .url
        .clone()
        .splitn(2, "//")
        .last()
        .expect("File should have a protocol")
        .to_string();

    if url.ends_with('/') {
        // We need to generate a file name, as there wasn't one given
        let guessed_ext = match mime_guess::get_mime_extensions_str(&resource.mime_type) {
            None => "txt",
            Some(mime_extensions) => mime_extensions
                .last()
                .expect("MIME returned no extensions in a Some; weird!"),
        };

        url.push_str("_unnamed_index.");
        url.push_str(guessed_ext);
    }

    let path = inside.join(&url);
    let parent_path = path.parent().expect("Could not get parent dir");
    println!("Writing file {:?}...", path);
    std::fs::create_dir_all(parent_path)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(&resource.data)
}

fn save_archive(archive: WebArchive, inside: &Path) -> std::io::Result<()> {
    println!("Saving main resource...");
    save(archive.main_resource, inside)?;

    if let Some(subresources) = archive.subresources {
        println!("Saving subresources...");
        subresources
            .into_iter()
            .for_each(|subresource| save(subresource, inside).expect("Could not save subresource"));
    }

    if let Some(subframe_archives) = archive.subframe_archives {
        println!("Saving subframe archives...");
        subframe_archives.into_iter().for_each(|subframe_archive| {
            save_archive(subframe_archive, inside).expect("Could not save subframe_archive")
        });
    }

    Ok(())
}

fn main() -> Result<()> {
    let options = Options::from_args();

    println!("{:?}", options);

    // TODO: Check file type, remove this "expect"
    let webarchive: WebArchive = webarchive::from_file(&options.input)
        .unwrap_or_else(|_| panic!("failed to read {:?}", options.input));

    let output = match &options.output {
        Some(path) => path,
        None => options
            .input
            .parent()
            .expect("Could not get an output directory"),
    };

    save_archive(webarchive, output).expect("could not save resources");

    Ok(())
}
