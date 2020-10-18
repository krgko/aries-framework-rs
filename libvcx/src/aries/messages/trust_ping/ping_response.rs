use agency_vcx::thread::Thread;
use aries::messages::a2a::{A2AMessage, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PingResponse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

impl PingResponse {
    pub fn create() -> PingResponse {
        PingResponse::default()
    }

    pub fn set_comment(mut self, comment: String) -> PingResponse {
        self.comment = Some(comment);
        self
    }
}

threadlike!(PingResponse);
a2a_message!(PingResponse);

#[cfg(test)]
pub mod tests {
    use aries::messages::connection::response::tests::*;

    use super::*;

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _ping_response() -> PingResponse {
        PingResponse {
            id: MessageId::id(),
            thread: _thread(),
            comment: Some(_comment()),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_ping_response_build_works() {
        let ping_response: PingResponse = PingResponse::default()
            .set_comment(_comment())
            .set_thread_id(&_thread_id());

        assert_eq!(_ping_response(), ping_response);
    }
}