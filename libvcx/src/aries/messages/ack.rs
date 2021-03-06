use messages::thread::Thread;
use aries::messages::a2a::{A2AMessage, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Ack {
    #[serde(rename = "@id")]
    pub id: MessageId,
    status: AckStatus,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AckStatus {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "PENDING")]
    Pending,
}

impl Default for AckStatus {
    fn default() -> AckStatus {
        AckStatus::Ok
    }
}

impl Ack {
    pub fn create() -> Ack {
        Ack::default()
    }

    pub fn set_status(mut self, status: AckStatus) -> Ack {
        self.status = status;
        self
    }
}

threadlike!(Ack);
a2a_message!(Ack);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PleaseAck {}

#[macro_export]
macro_rules! please_ack (($type:ident) => (
    impl $type {
        pub fn ask_for_ack(mut self) -> $type {
            self.please_ack = Some(PleaseAck {});
            self
        }
    }
));

#[cfg(test)]
pub mod tests {
    use aries::messages::connection::response::tests::*;

    use super::*;

    pub fn _ack() -> Ack {
        Ack {
            id: MessageId::id(),
            status: AckStatus::Fail,
            thread: _thread(),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_ack_build_works() {
        let ack: Ack = Ack::default()
            .set_status(AckStatus::Fail)
            .set_thread_id(&_thread_id());

        assert_eq!(_ack(), ack);
    }
}