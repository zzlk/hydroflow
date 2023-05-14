// mod serialization;

#[cfg(test)]
mod test;

// pub use serialization::KvsRequestDeserializer;

use bytes::Bytes;
use lattices::bottom::Bottom;
use lattices::map_union::MapUnionHashMap;
use lattices::set_union::SetUnionHashSet;
use lattices::{dom_pair::DomPair, fake::Fake, ord::Max};
use serde::de::{DeserializeOwned, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;

pub type NodeId = usize;

pub type MyLastWriteWins = DomPair<Max<u128>, Bottom<Fake<BytesWrapper>>>;
pub type MySetUnion = SetUnionHashSet<(NodeId, usize)>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KvsRequest {
    Put {
        key: u64,
        value: BytesWrapper,
    },
    Get {
        key: u64,
    },
    Gossip {
        map: MapUnionHashMap<u64, MyLastWriteWins>,
    },
    Delete {
        key: u64,
    },
}

#[derive(Clone, Debug)]
pub enum KvsResponse {
    _PutResponse { key: u64 },
    GetResponse { key: u64, reg: MyLastWriteWins },
}

#[derive(Clone, Debug)]
pub struct BytesWrapper(pub Bytes);

impl Serialize for BytesWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for BytesWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl Visitor<'_> for V {
            type Value = Bytes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(std::any::type_name::<Self::Value>())
            }

            fn visit_bytes_bytes<E>(self, v: Bytes) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v)
            }
        }

        Ok(BytesWrapper(deserializer.deserialize_bytes(V)?))
    }
}
