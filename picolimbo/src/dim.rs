use std::io::Cursor;

use anyhow::bail;
use picolimbo_proto::nbt::{self, Blob, Value};

lazy_static::lazy_static! {
    pub static ref DIMENSION_MANAGER: DimensionManager = DimensionManager::init();
}

#[derive(Debug, Clone)]
pub struct DimensionManager {
    pub codec_1_16: nbt::Blob,
    pub codec_1_18_2: nbt::Blob,
    pub codec_1_19: nbt::Blob,
    pub codec_1_19_1: nbt::Blob,
    pub codec_1_19_4: nbt::Blob,
    pub codec_legacy: nbt::Blob,
}

impl DimensionManager {
    pub fn init() -> DimensionManager {
        let legacy = include_bytes!("../res/codecs/codec_legacy.nbt");
        let codec_1_16 = include_bytes!("../res/codecs/codec_1_16.nbt");
        let codec_1_18_2 = include_bytes!("../res/codecs/codec_1_18_2.nbt");
        let codec_1_19 = include_bytes!("../res/codecs/codec_1_19.nbt");
        let codec_1_19_1 = include_bytes!("../res/codecs/codec_1_19_1.nbt");
        let codec_1_19_4 = include_bytes!("../res/codecs/codec_1_19_4.nbt");

        DimensionManager {
            codec_1_16: nbt::Blob::from_gzip_reader(&mut Cursor::new(codec_1_16)).unwrap(),
            codec_1_18_2: nbt::Blob::from_gzip_reader(&mut Cursor::new(codec_1_18_2)).unwrap(),
            codec_1_19: nbt::Blob::from_gzip_reader(&mut Cursor::new(codec_1_19)).unwrap(),
            codec_1_19_1: nbt::Blob::from_gzip_reader(&mut Cursor::new(codec_1_19_1)).unwrap(),
            codec_1_19_4: nbt::Blob::from_gzip_reader(&mut Cursor::new(codec_1_19_4)).unwrap(),
            codec_legacy: nbt::Blob::from_gzip_reader(&mut Cursor::new(legacy)).unwrap(),
        }
    }

    pub fn default_dim_for_codec(
        &self,
        preferred: &String,
        codec: &Blob,
    ) -> anyhow::Result<Dimension> {
        if let Value::Compound(dim_cmp) = codec.get("minecraft:dimension_type").unwrap() {
            if let Value::List(list) = dim_cmp.get("value").unwrap() {
                match &**preferred {
                    "the_nether" => {
                        let ov = list.get(2).unwrap().clone();
                        return Ok(Dimension {
                            id: -1,
                            name: "minecraft:nether".to_string(),
                            data: ov,
                        });
                    }
                    "the_end" => {
                        let ov = list.get(3).unwrap().clone();
                        return Ok(Dimension {
                            id: 1,
                            name: "minecraft:the_end".to_string(),
                            data: ov,
                        });
                    }
                    _ => {
                        let ov = list.get(0).unwrap().clone();
                        return Ok(Dimension {
                            id: 0,
                            name: "minecraft:overworld".to_string(),
                            data: ov,
                        });
                    }
                }
            }
        }
        bail!("Invalid codec data")
    }

    pub fn default_dim_1_16(&self, preferred: &String) -> anyhow::Result<Dimension> {
        self.default_dim_for_codec(preferred, &self.codec_1_16)
    }

    pub fn default_dim_1_18_2(&self, preferred: &String) -> anyhow::Result<Dimension> {
        self.default_dim_for_codec(preferred, &self.codec_1_18_2)
    }
}

#[derive(Debug, Clone)]
pub struct Dimension {
    pub id: i8,
    pub name: String,
    pub data: nbt::Value,
}
