use webarchive::WebArchive;

fn main() {
    let webarchive: WebArchive = webarchive::from_file("psxdatacenter2.webarchive")
        .expect("failed to read psxdatacenter2.webarchive");

    webarchive.print_list();
}
