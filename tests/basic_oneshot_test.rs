use std::sync::atomic::{AtomicUsize, Ordering};
use swiftbots::BasicBot;
use tokio;
use swiftbots::types::SwiftBotsError;

struct Bot1 {
    counter: AtomicUsize
}

#[async_trait::async_trait]
impl BasicBot for Bot1 {
    type Message = ();

    fn name(&self) -> &str { "" }

    fn enabled(&self) -> bool {
        false
    }

    async fn listen(&self) -> Self::Message {
        ()
    }

    async fn handle(&self, _: Self::Message) {
        self.counter.fetch_add(111, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn base_handler_test() -> Result<(), SwiftBotsError>{
    let bot = Bot1 {
        counter: AtomicUsize::new(0)
    };
    bot.handle(()).await;

    assert_eq!(bot.counter.load(Ordering::Relaxed), 111);
    Ok(())
}