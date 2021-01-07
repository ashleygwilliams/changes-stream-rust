use changes_stream::ChangesStream;

#[tokio::main]
async fn main() {
    let url = String::from("https://replicate.npmjs.com/_changes");
    let mut changes = ChangesStream::new(url);
    changes.on(|change| {
        println!("{}: {}", change.seq, change.id);
    });
    changes.run().await;
}
