use webarchive::Webarchive;

fn main() {
    let webarchive: Webarchive = plist::from_file("minidisc.webarchive")
        .expect("failed to read minidisc.webarchive");

    println!("{:?}", webarchive);
}
