use crate::event_bot::bot::EventBot;


pub trait ChatBot: EventBot {
    fn message_handlers(&self) -> Vec<dyn MessageHandler<Self::Message>>;
}

pub trait MessageHandler<T> {
    fn commands() -> Vec<&'static str>;
    async fn handle(&self, message: T, command: &str);
}