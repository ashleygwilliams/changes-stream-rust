use changes_stream2::{ChangesStream, Event};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() {
    let url = "https://replicate.npmjs.com/_changes".to_string();
    let mut changes = ChangesStream::new(url).await.unwrap();
    while let Some(event) = changes.next().await {
        match event {
            Ok(Event::Change(change)) => println!("Change ({}): {}", change.seq, change.id),
            Ok(Event::Finished(finished)) => println!("Finished: {}", finished.last_seq),
            Err(err) => println!("Error: {:?}", err),
        }
    }
}
