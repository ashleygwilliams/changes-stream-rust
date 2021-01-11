use changes_stream::{ChangesStream, Event};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() {
    let url = "https://replicate.npmjs.com/_changes".to_string();
    let mut changes = ChangesStream::new(url).await;
    while let Some(event) = changes.next().await {
        match event {
            Event::Change(change) => {
                println!("{}: {}", change.seq, change.id);
            }
            Event::Finished(finished) => {
                println!("Finished: {}", finished.last_seq);
            }
        }
    }
}
