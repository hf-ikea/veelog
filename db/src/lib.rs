pub mod data;
pub mod util;

pub(crate) const VEELOG_MAGIC: &[u8; 32] = b"D784CB9E58D279B42FDA4D0A5FC7DA80";

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::remove_dir_all,
        panic::UnwindSafe,
        path::Path,
        thread,
        time::Duration,
    };

    use crate::data::{FieldType, Log, LogHeader, LogRecord};
    use sled::Db;

    #[test]
    pub fn db_playground() {
        test_with_db(|db| {
            let header = LogHeader::new("N0CALL", "");
            let mut log = Log::new_init(db, header).unwrap();

            log.import_adif_file("../testlog2.adi".into()).unwrap();

            for record in log.get_records() {
                for f in record.iter() {
                    println!("{} {}", f.0, f.1);
                }
            }
        });
    }

    #[test]
    pub fn test_db() {
        test_with_db(|db| {
            let header = LogHeader::new("N0CALL", "");
            let mut testlog = Log::new_init(db, header).unwrap();
            let mut record = LogRecord::new();
            record
                .insert_field(FieldType::WorkedCall, "N0CALL")
                .insert_field(FieldType::GridSquare, "AA00")
                .insert_timestamp("2025-07-28T02:48:13Z".parse().unwrap());

            testlog.insert_record(record).unwrap();
            let dec = testlog.get_record(0).unwrap();

            assert_eq!(
                "N0CALL".to_string(),
                dec.get_field(&FieldType::WorkedCall).unwrap()
            );
            assert_eq!(
                "AA00".to_string(),
                dec.get_field(&FieldType::GridSquare).unwrap()
            );
            assert_eq!(
                "2025-07-28T02:48:13Z".to_string(),
                dec.get_field(&FieldType::Timestamp).unwrap()
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
