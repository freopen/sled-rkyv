#[cfg(test)]
mod tests {

    use rkyv::{Archive, Deserialize, Serialize};
    use sled_rkyv::Collection;

    fn init() {
        let config = sled_rkyv::Config::new().path(tempfile::tempdir().unwrap());
        sled_rkyv::set_config(config);
    }

    #[derive(Archive, Clone, Debug, Serialize, Deserialize, Collection, PartialEq)]
    struct EmptyId {
        string: String,
        number: i32,
    }

    #[test]
    fn empty_id() {
        init();
        let msg = EmptyId {
            string: "test".into(),
            number: 4,
        };
        assert_eq!(None, EmptyId::get(&()).unwrap());
        assert_eq!(None, msg.insert().unwrap());
        let from_db = EmptyId::get(&()).unwrap().unwrap();
        assert_eq!(4, from_db.number);
        assert_eq!(
            msg,
            from_db.to_archive().unwrap()
        );
        assert_eq!(
            msg,
            EmptyId::remove(&()).unwrap().unwrap().to_archive().unwrap()
        );
        assert_eq!(None, EmptyId::get(&()).unwrap());
    }

    #[derive(Archive, Clone, Debug, Serialize, Deserialize, Collection, PartialEq)]
    struct StringId {
        #[key] string: String,
        number: i32,
    }

    #[test]
    fn string_id() {
        init();
        let msg = StringId {
            string: "test".into(),
            number: 4,
        };
        assert_eq!(None, StringId::get("test").unwrap());
        assert_eq!(None, StringId::get("non-existing").unwrap());
        assert_eq!(None, msg.insert().unwrap());
        assert_eq!(
            msg,
            StringId::get("test").unwrap().unwrap().to_archive().unwrap()
        );
        assert_eq!(None, StringId::get("TEST").unwrap());
        assert_eq!(
            msg,
            StringId::remove("test").unwrap().unwrap().to_archive().unwrap()
        );
        assert_eq!(None, StringId::get("test").unwrap());
        assert_eq!(None, StringId::get("non-existing").unwrap());
    }

    #[derive(Archive, Clone, Debug, Serialize, Deserialize, Collection, PartialEq)]
    struct CaseInsensitiveId {
        #[key(case_insensitive)] string: String,
        number: i32,
    }

    #[test]
    fn case_insensitive_id() {
        init();
        let msg = CaseInsensitiveId {
            string: "test".into(),
            number: 4,
        };
        assert_eq!(None, CaseInsensitiveId::get("test").unwrap());
        assert_eq!(None, CaseInsensitiveId::get("non-existing").unwrap());
        assert_eq!(None, msg.insert().unwrap());
        assert_eq!(
            msg,
            CaseInsensitiveId::get("TEST").unwrap().unwrap().to_archive().unwrap()
        );
        assert_eq!(None, CaseInsensitiveId::get("non-existing").unwrap());
        assert_eq!(
            msg,
            CaseInsensitiveId::remove("TeSt").unwrap().unwrap().to_archive().unwrap()
        );
        assert_eq!(None, CaseInsensitiveId::get("tEsT").unwrap());
        assert_eq!(None, CaseInsensitiveId::get("non-existing").unwrap());
    }
}
