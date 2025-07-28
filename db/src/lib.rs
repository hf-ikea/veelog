pub mod data;

pub(crate) const VEELOG_MAGIC: &[u8; 32] = b"D784CB9E58D279B42FDA4D0A5FC7DA80";

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self, remove_dir_all},
        panic::UnwindSafe,
        path::Path,
        thread,
        time::Duration,
    };

    use crate::{
        data::{FieldType, Log, LogRecord},
    };
    use adif::parse;
    use sled::Db;

    #[test]
    pub fn db_playground() {
        test_with_db(|db| {
            let mut log = Log::new(db).unwrap();

            let data: String = fs::read_to_string("../testlog.adi").unwrap();
            let adif = parse::parse_adif(&data);

            log.import_adif(adif).unwrap();

            for i in 0..log.get_len() {
                let record = log.get_record(i).unwrap();
                for f in record.into_iter() {
                    println!("{} {}", f.0, f.1);
                }
            }
        });
    }

    #[test]
    pub fn test_db() {
        test_with_db(|db| {
            let mut testlog = Log::new(db).unwrap();
            let mut record = LogRecord::new();
            record
                .insert_field(FieldType::WorkedCall, "N0CALL")
                .insert_field(FieldType::GridSquare, "AA00")
                .insert_timestamp("2025-07-28T02:48:13Z".parse().unwrap());

            testlog.insert_record(record).unwrap();
            let dec = testlog.get_record(0).unwrap();

            assert_eq!(
                "N0CALL".to_string(),
                dec.get_field(FieldType::WorkedCall).unwrap()
            );
            assert_eq!(
                "AA00".to_string(),
                dec.get_field(FieldType::GridSquare).unwrap()
            );
            assert_eq!(
                "2025-07-28T02:48:13Z".to_string(),
                dec.get_field(FieldType::Timestamp).unwrap()
            );
        });
    }

    fn test_with_db(test: impl FnOnce(Db) + UnwindSafe) {
        let path = env::temp_dir().join(Path::new("veelog-tests-db"));
        let _ = remove_dir_all(&path);

        let db = sled::open(&path).unwrap();

        test(db);

        thread::sleep(Duration::from_millis(100));

        remove_dir_all(&path).unwrap();
    }
}
