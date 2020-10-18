use serde_json::Value;

use error::{VcxError, VcxErrorKind, VcxResult};
use messages::get_message::MessagePayload;
use messages::message_type::*;
use messages::thread::Thread;
use settings::{get_protocol_type, ProtocolTypes};
use utils::libindy::crypto;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Payloads {
    PayloadV1(PayloadV1),
    PayloadV2(PayloadV2),
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct PayloadV1 {
    #[serde(rename = "@type")]
    pub type_: PayloadTypeV1,
    #[serde(rename = "@msg")]
    pub msg: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct PayloadV2 {
    #[serde(rename = "@type")]
    pub type_: PayloadTypeV2,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@msg")]
    pub msg: String,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

impl Payloads {
    // TODO: Refactor Error
    // this will become a CommonError, because multiple types (Connection/Issuer Credential) use this function
    // Possibly this function moves out of this file.
    // On second thought, this should stick as a ConnectionError.
    pub fn encrypt(my_vk: &str, their_vk: &str, data: &str, msg_type: PayloadKinds, thread: Option<Thread>) -> VcxResult<Vec<u8>> {
        match ProtocolTypes::from(get_protocol_type().to_string()) {
            ProtocolTypes::V1 |
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 => {
                let thread = thread.ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Thread info not found"))?;

                let payload = PayloadV2 {
                    type_: PayloadTypes::build_v2(msg_type),
                    id: String::new(),
                    msg: data.to_string(),
                    thread,
                };

                let message = ::serde_json::to_string(&payload)
                    .map_err(|err| {
                        error!("could not serialize create_keys msg: {}", err);
                        VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize payload: {}", err))
                    })?;

                let receiver_keys = ::serde_json::to_string(&vec![&their_vk])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize receiver keys: {}", err)))?;

                trace!("Sending payload: {:?}", message.as_bytes());
                crypto::pack_message(Some(my_vk), &receiver_keys, message.as_bytes())
            }
        }
    }

    pub fn decrypt(my_vk: &str, payload: &MessagePayload) -> VcxResult<(String, Option<Thread>)> {
        match payload {
            MessagePayload::V2(payload) => {
                let payload = Payloads::decrypt_payload_v2(my_vk, payload)?;
                Ok((payload.msg, Some(payload.thread)))
            }
        }
    }

    pub fn decrypt_payload_v2(_my_vk: &str, payload: &::serde_json::Value) -> VcxResult<PayloadV2> {
        let payload = ::serde_json::to_vec(&payload)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, err))?;

        let unpacked_msg = crypto::unpack_message(&payload)?;

        let message: ::serde_json::Value = ::serde_json::from_slice(unpacked_msg.as_slice())
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize payload: {}", err)))?;

        let message = message["message"].as_str()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, "Cannot find `message` field"))?.to_string();

        let mut my_payload: PayloadV2 = serde_json::from_str(&message)
            .map_err(|err| {
                error!("could not deserialize bundle with i8 or u8: {}", err);
                VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize payload: {}", err))
            })?;

        if my_payload.thread.thid.is_none() {
            my_payload.thread.thid = Some(my_payload.id.clone());
        }

        Ok(my_payload)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PayloadTypes {
    PayloadTypeV1(PayloadTypeV1),
    PayloadTypeV2(PayloadTypeV2),
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct PayloadTypeV1 {
    pub name: String,
    ver: String,
    fmt: String,
}

type PayloadTypeV2 = MessageTypeV2;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PayloadKinds {
    CredOffer,
    CredReq,
    Cred,
    Proof,
    ProofRequest,
    Other(String),
}

impl PayloadKinds {
    fn family(&self) -> MessageFamilies {
        match self {
            PayloadKinds::CredOffer => MessageFamilies::CredentialExchange,
            PayloadKinds::CredReq => MessageFamilies::CredentialExchange,
            PayloadKinds::Cred => MessageFamilies::CredentialExchange,
            PayloadKinds::Proof => MessageFamilies::CredentialExchange,
            PayloadKinds::ProofRequest => MessageFamilies::CredentialExchange,
            PayloadKinds::Other(family) => MessageFamilies::Unknown(family.to_string()),
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        match get_protocol_type() {
            ProtocolTypes::V1 => {
                match self {
                    PayloadKinds::CredOffer => "CRED_OFFER",
                    PayloadKinds::CredReq => "CRED_REQ",
                    PayloadKinds::Cred => "CRED",
                    PayloadKinds::ProofRequest => "PROOF_REQUEST",
                    PayloadKinds::Proof => "PROOF",
                    PayloadKinds::Other(kind) => kind,
                }
            }
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 => {
                match self {
                    PayloadKinds::CredOffer => "credential-offer",
                    PayloadKinds::CredReq => "credential-request",
                    PayloadKinds::Cred => "credential",
                    PayloadKinds::ProofRequest => "presentation-request",
                    PayloadKinds::Proof => "presentation",
                    PayloadKinds::Other(kind) => kind,
                }
            }
        }
    }
}

impl PayloadTypes {
    pub fn build_v1(kind: PayloadKinds, fmt: &str) -> PayloadTypeV1 {
        PayloadTypeV1 {
            name: kind.name().to_string(),
            ver: MESSAGE_VERSION_V1.to_string(),
            fmt: fmt.to_string(),
        }
    }

    pub fn build_v2(kind: PayloadKinds) -> PayloadTypeV2 {
        PayloadTypeV2 {
            did: DID.to_string(),
            family: kind.family(),
            version: kind.family().version().to_string(),
            type_: kind.name().to_string(),
        }
    }
}