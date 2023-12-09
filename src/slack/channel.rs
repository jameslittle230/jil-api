use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, Default, PartialEq, Eq, Clone, Debug)]
pub(crate) enum SlackChannel {
    #[default]
    #[strum(to_string = "C75C3AW66", serialize = "general")]
    General,

    #[strum(to_string = "CLVH6SLAZ", serialize = "jil-dot-im")]
    JilDotIm,

    #[strum(to_string = "CVBH1GHSM", serialize = "jil-guestbook")]
    JilGuestbook,

    #[strum(to_string = "C01HFPUJGHZ", serialize = "rrl-feedback")]
    RRLFeedback,

    #[strum(to_string = "C05QT3QPDNY", serialize = "wedding-site")]
    WeddingSite,

    #[strum(to_string = "C069AFX67C5", serialize = "lights")]
    Lights,

    Unknown(String),
}

impl Serialize for SlackChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Self::Unknown(s) = self {
            return serializer.serialize_str(s);
        }

        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SlackChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_str(&s).unwrap_or(Self::Unknown(s)))
    }
}

// impl From<String> for SlackChannel {
//     fn from(value: String) -> Self {
//         Self::from_str(&value).unwrap_or(Self::Unknown(value))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_channel_deserialize() {
        let channel: SlackChannel = serde_json::from_str("\"general\"").unwrap();
        assert_eq!(channel, SlackChannel::General);
    }

    #[test]
    fn test_slack_channel_deserialize_wedding_site() {
        let channel: SlackChannel = serde_json::from_str("\"wedding-site\"").unwrap();
        assert_eq!(channel, SlackChannel::WeddingSite);
    }

    #[test]
    fn test_default_slack_channel_general() {
        let channel: SlackChannel = SlackChannel::default();
        assert_eq!(channel, SlackChannel::General);
    }

    #[test]
    fn test_unknown_slack_channel_deserialize() {
        let channel: SlackChannel = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(channel, SlackChannel::Unknown("unknown".to_string()));
    }

    #[test]
    fn test_slack_channel_serialize() {
        let channel = SlackChannel::General;
        let serialized = serde_json::to_string(&channel).unwrap();
        assert_eq!(serialized, "\"C75C3AW66\"");
    }

    #[test]
    fn test_slack_channel_serialize_unknown() {
        let channel = SlackChannel::Unknown("unknownABC".to_string());
        let serialized = serde_json::to_string(&channel).unwrap();
        assert_eq!(serialized, "\"unknownABC\"");
    }
}
