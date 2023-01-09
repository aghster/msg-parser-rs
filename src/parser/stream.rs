use crate::ole::EntrySlice;

use super::{
    constants::PropIdNameMap,
    decode::{DataType, PtypDecoder},
    storage::StorageType,
};

// StreamType
#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    SubStorage,
    PropertyStream,
}

// Stream refer to an element in Message object.
#[derive(Debug, PartialEq)]
pub struct Stream {
    // Storage that a stream belongs to
    pub parent: StorageType,
    pub key: String,
    pub value: DataType,
    pub _type: StreamType,
}

impl Stream {
    // __substg1.0__AAAABBBB where AAAA is property id and BBBB is property datatype
    fn extract_id_and_datatype(name: &str) -> (String, String) {
        let tag = name
            .split("_")
            .filter(|&x| x.len() > 0)
            .collect::<Vec<&str>>()[1];
        let prop_id = String::from("0x") + &tag[..4];
        let prop_datatype = String::from("0x") + &tag[tag.len() - 4..];
        return (prop_id, prop_datatype);
    }

    fn stream_type(name: &str) -> Option<StreamType> {
        match name {
            "__properties_version1.0" => Some(StreamType::PropertyStream),
            _ if name.starts_with("__substg1.0") => Some(StreamType::SubStorage),
            _ => None,
        }
    }

    pub fn create(
        name: &str,
        entry_slice: &mut EntrySlice,
        prop_map: &PropIdNameMap,
        parent: &StorageType,
    ) -> Option<Self> {
        match Self::stream_type(name) {
            Some(StreamType::SubStorage) => {
                // Split name up into property id and datatype
                let (prop_id, prop_datatype) = Self::extract_id_and_datatype(name);
                let key = prop_map.get_canonical_name(&prop_id)?;
                let value_res = PtypDecoder::decode(entry_slice, &prop_datatype);
                if value_res.is_err() {
                    return None;
                }
                let value = value_res.unwrap();
                return Some(Self {
                    parent: parent.clone(),
                    key,
                    value,
                    _type: StreamType::SubStorage,
                })
            },
            Some(StreamType::PropertyStream) => {
                let value_res = PtypDecoder::decode(entry_slice, "0x0102");
                if value_res.is_err() {
                    return None;
                }
                let value = value_res.unwrap();
                return Some(Self {
                    parent: parent.clone(),
                    key: String::from(name),
                    value,
                    _type: StreamType::PropertyStream,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::constants::PropIdNameMap, super::decode::DataType, super::storage::StorageType,
        Stream, StreamType,
    };
    use crate::ole::Reader;

    #[test]
    fn test_extract_id_and_datatype() {
        let (prop_id, prop_datatype) = Stream::extract_id_and_datatype("__substg1.0_3701000D");
        assert_eq!(prop_id, "0x3701");
        assert_eq!(prop_datatype, "0x000D");

        let (prop_id, prop_datatype) = Stream::extract_id_and_datatype("__substg1.0_1016102F");
        assert_eq!(prop_id, "0x1016");
        assert_eq!(prop_datatype, "0x102F");
    }

    #[test]
    fn test_stream_type() {
        assert_eq!(Stream::stream_type("__recip_version1.0_#00000000"), None);
        assert_eq!(Stream::stream_type("__substg1.0_3701000D"), Some(StreamType::SubStorage));
        assert_eq!(Stream::stream_type("__properties_version1.0"), Some(StreamType::PropertyStream));
    }

    #[test]
    fn test_create_stream() {
        let parser = Reader::from_path("data/test_email.msg").unwrap();
        let prop_map = PropIdNameMap::init();

        // Root entry is ok.
        let mut slice = parser
            .iterate()
            .filter(|x| x.name() == "__substg1.0_0C1F001F")
            .nth(0)
            .and_then(|entry| parser.get_entry_slice(entry).ok())
            .unwrap();

        let stream = Stream::create(
            "__substg1.0_0C1F001F",
            &mut slice,
            &prop_map,
            &StorageType::RootEntry,
        );
        assert_eq!(
            stream,
            Some(Stream {
                key: "SenderEmailAddress".to_string(),
                value: DataType::PtypString("upgrade@asuswebstorage.com".to_string()),
                parent: StorageType::RootEntry,
                _type: StreamType::SubStorage,
            })
        );

        // Recipient object check.
        let mut slice = parser
            .iterate()
            .filter(|x| x.name() == "__substg1.0_3001001F")
            .nth(0)
            .and_then(|entry| parser.get_entry_slice(entry).ok())
            .unwrap();
        let stream = Stream::create(
            "__substg1.0_3001001F",
            &mut slice,
            &prop_map,
            &StorageType::Recipient(1),
        );
        assert_eq!(
            stream,
            Some(Stream {
                key: "DisplayName".to_string(),
                value: DataType::PtypString("Sriram Govindan".to_string()),
                parent: StorageType::Recipient(1),
                _type: StreamType::SubStorage,
            })
        )
    }

    #[test]
    fn test_create_attachment() {
        let parser = Reader::from_path("data/attachment.msg").unwrap();
        let prop_map = PropIdNameMap::init();

        // Attachment object.
        let mut attachment = parser
            .iterate()
            .find(|x| x.name() == "__substg1.0_3703001F" && x.parent_node() == Some(7u32))
            .and_then(|entry| parser.get_entry_slice(entry).ok())
            .unwrap();
        let stream = Stream::create(
            "__substg1.0_3703001F",
            &mut attachment,
            &prop_map,
            &StorageType::Attachment(0),
        );
        assert_eq!(
            stream,
            Some(Stream {
                key: "AttachExtension".to_string(),
                value: DataType::PtypString(".doc".to_string()),
                parent: StorageType::Attachment(0),
                _type: StreamType::SubStorage,
            })
        )
    }
}
