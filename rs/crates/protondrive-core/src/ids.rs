use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! opaque_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }
    };
}

opaque_id!(VolumeId,   "Opaque identifier for a Proton Drive volume");
opaque_id!(ShareId,    "Opaque identifier for a Proton Drive share");
opaque_id!(NodeId,     "Opaque identifier for a file or folder node");
opaque_id!(RevisionId, "Opaque identifier for a file revision");
opaque_id!(EventId,    "Opaque identifier for an event stream position");
opaque_id!(LinkId,     "Opaque identifier for a public share link");
opaque_id!(AlbumId,    "Opaque identifier for a photo album");
