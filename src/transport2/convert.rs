pub trait MessageConvert {
    type Message;

    fn extract_text(msg: Self::Message) -> Option<String>;
    fn create_message(text: String) -> Self::Message;
}

#[derive(Debug)]
pub struct Tungstenite;

impl MessageConvert for Tungstenite {
    type Message = tokio_tungstenite::tungstenite::Message;

    fn extract_text(msg: Self::Message) -> Option<String> {
        if let Self::Message::Text(s) = msg {
            Some(s)
        } else {
            None
        }
    }

    fn create_message(text: String) -> Self::Message {
        Self::Message::Text(text)
    }
}
