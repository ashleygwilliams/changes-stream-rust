extern crate changes_stream;

use changes_stream::ChangesStream;

#[test]
fn test_new() {
  let db = String::from("https://replciate.npmjs.com/_changes");
  let changes = ChangesStream::new(db);
}
