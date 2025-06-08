use super::Shutter;

use bincode::config::BigEndian;

use bincode::config::Configuration;

const BINCODE_CONFIG: Configuration<BigEndian> = bincode::config::standard().with_big_endian();

pub struct Database {
    kv: sled::Db,
}

impl Database {
    pub fn new() -> Self {
        let kv = sled::open("database").unwrap();
        Self { kv }
    }

    pub fn generate_id(&self) -> u64 {
        self.kv.generate_id().unwrap() as u64
    }

    pub fn insert_shutter(&self, shutter: Shutter) -> anyhow::Result<()> {
        let key = [*b"shutter_", shutter.id.to_be_bytes()].concat();
        self.kv
            .insert(key, bincode::encode_to_vec(shutter, BINCODE_CONFIG)?)?;
        Ok(())
    }

    pub fn remove_shutter(&self, id: u64) -> anyhow::Result<()> {
        let key = [*b"shutter_", id.to_be_bytes()].concat();
        self.kv.remove(key)?;
        Ok(())
    }

    pub fn get_shutter(&self, id: u64) -> anyhow::Result<Option<Shutter>> {
        let key = [*b"shutter_", id.to_be_bytes()].concat();
        Ok(self
            .kv
            .get(key)?
            .map(|b| bincode::decode_from_slice(&b, BINCODE_CONFIG).unwrap())
            .map(|(s, _)| s))
    }

    pub fn iter_shutters(&self) -> impl Iterator<Item = Shutter> {
        self.kv
            .scan_prefix(b"shutter_")
            .filter_map(|r| Some(r.unwrap()))
            .map(|(_, v)| bincode::decode_from_slice(&v, BINCODE_CONFIG).unwrap())
            .map(|(s, _)| s)
    }
}
