use changes_stream::ChangesStream;

#[test]
fn test_new() {
    let db = String::from("https://replicate.npmjs.com/_changes");
    let _changes = ChangesStream::new(db);
}
