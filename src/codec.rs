pub trait MessageCodec {
    type Message;

    fn decode(msg: Self::Message) -> Option<String>;
    fn encode(text: String) -> Self::Message;
}

#[derive(Debug)]
pub struct TungsteniteCodec;

impl MessageCodec for TungsteniteCodec {
    type Message = tokio_tungstenite::tungstenite::Message;

    fn decode(msg: Self::Message) -> Option<String> {
        if let Self::Message::Text(s) = msg {
            Some(s)
        } else {
            None
        }
    }

    fn encode(text: String) -> Self::Message {
        Self::Message::Text(text)
    }
}
