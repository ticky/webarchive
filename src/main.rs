use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use webarchive::{WebArchive, WebResource};

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
    let parent_path = path.parent().expect("Could not get parent directory");

    println!("Writing file {:?}...", path);

    std::fs::create_dir_all(parent_path)?;
    std::fs::File::create(path)?.write_all(&resource.data)
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

#[derive(Debug, StructOpt)]
enum Args {
    /// List the contents of a webarchive file
    Inspect {
        #[structopt(parse(from_os_str))]
        /// File or folder to inspect
        input: PathBuf,
    },

    /// Extract the contents of a webarchive file to individual files
    Extract {
        #[structopt(parse(from_os_str))]
        /// File or folder to convert
        input: PathBuf,

        #[structopt(short, long, parse(from_os_str))]
        /// File or folder name to output to.
        ///
        /// If omitted, files will be written to
        /// the folder containing the input file.
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let args = Args::from_args();

    match args {
        Args::Inspect { input } => {
            let webarchive: WebArchive = webarchive::from_file(&input)
                .with_context(|| format!("failed to read {:?}", input))?;

            webarchive.print_list();

            Ok(())
        }

        Args::Extract { input, output } => {
            let webarchive: WebArchive = webarchive::from_file(&input)
                .with_context(|| format!("failed to read {:?}", input))?;

            let output = match &output {
                Some(path) => path,
                None => input
                    .parent()
                    .context("Could not get an output directory")?,
            };

            save_archive(webarchive, output).context("Saving resources")
        }
    }
}
