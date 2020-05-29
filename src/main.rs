use webarchive::Webarchive;

fn main() {
    let webarchive: Webarchive = plist::from_file("psxdatacenter.webarchive")
        .expect("failed to read psxdatacenter.webarchive");

    webarchive.print_list();
}
