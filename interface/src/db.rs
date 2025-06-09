use super::Shutter;

use bincode::config::BigEndian;

use bincode::config::Configuration;
use sled::IVec;

const BINCODE_CONFIG: Configuration<BigEndian> = bincode::config::standard().with_big_endian();

pub struct Database {
    kv: sled::Db,
}

impl Database {
    pub fn new() -> Self {
        let kv = sled::open("database").unwrap();
        Self { kv }
    }

    fn primary_key(id: u64) -> IVec {
        [*b"shutter_", id.to_be_bytes()].concat().into()
    }

    // fn lookup_key(topic: &str) -> IVec {
    //     let prefix = "lookup_";
    //     let mut vec = Vec::with_capacity(prefix.len() + topic.len());
    //     vec.extend_from_slice(prefix.as_bytes());
    //     vec.extend_from_slice(topic.as_bytes());
    //     vec.into()
    //     // [*b"lookup_", topic.as_bytes()].concat().into()
    // }

    pub fn generate_id(&self) -> u64 {
        self.kv.generate_id().unwrap() as u64
    }

    pub fn insert_shutter(&self, shutter: Shutter) -> anyhow::Result<()> {
        let key = Self::primary_key(shutter.id);

        self.kv
            .insert(key, bincode::encode_to_vec(shutter, BINCODE_CONFIG)?)?;
        Ok(())
    }

    pub fn remove_shutter(&self, id: u64) -> anyhow::Result<()> {
        let key = Self::primary_key(id);
        self.kv.remove(key)?;
        Ok(())
    }

    pub fn get_shutter(&self, id: u64) -> anyhow::Result<Option<Shutter>> {
        let key = Self::primary_key(id);
        Ok(self
            .kv
            .get(key)?
            .map(|b| bincode::decode_from_slice(&b, BINCODE_CONFIG).unwrap())
            .map(|(s, _)| s))
    }

    pub fn get_shutter_by_status_topic(&self, topic: &str) -> Option<Shutter> {
        self.iter_shutters().find(|s| s.status_topic == topic)
    }

    pub fn iter_shutters(&self) -> impl Iterator<Item = Shutter> {
        self.kv
            .scan_prefix(b"shutter_")
            .filter_map(|r| Some(r.unwrap()))
            .map(|(_, v)| bincode::decode_from_slice(&v, BINCODE_CONFIG).unwrap())
            .map(|(s, _)| s)
    }
}
