use std::path::PathBuf;
use structopt::StructOpt;
use webarchive::WebArchive;

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

fn main() {
    let options = Options::from_args();

    println!("{:?}", options);

    // TODO: Check file type, remove this "expect"
    let webarchive: WebArchive = webarchive::from_file(&options.input)
        .expect(format!("failed to read {:?}", options.input).as_str());

    webarchive.print_list();
}
