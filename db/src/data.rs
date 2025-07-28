use adif::data::{ADIFFile, SerializeError};
use bincode::{
    Decode, Encode,
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
};
use jiff::{civil::{Date, Time}, fmt::strtime, tz::TimeZone, Timestamp};
use sled::Db;
use std::{collections::HashMap, fmt::Display};

use crate::{VEELOG_MAGIC, data};

#[derive(Debug)]
pub struct LogError {
    pub message: String,
    pub offender: String,
}

impl Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}. Offending value: {}", self.message, self.offender)
    }
}

impl From<SerializeError> for LogError {
    fn from(value: SerializeError) -> Self {
        LogError {
            message: value.message,
            offender: value.offender,
        }
    }
}

impl From<jiff::Error> for LogError {
    fn from(value: jiff::Error) -> Self {
        LogError {
            message: value.to_string(),
            offender: "".to_string(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    Timestamp,
    WorkedCall,
    Frequency,
    Mode,
    SentRST,
    RcvdRST,
    GridSquare,
    PrimaryAdminSubdiv,
    SentSerial,
    RcvdSerial,
    Comment,
    Other(Box<str>),
}

impl FieldType {
    pub fn from_adif_field(field_name: &str) -> Self {
        match field_name {
            "CALL" => Self::WorkedCall,
            "FREQ" => Self::Frequency,
            "MODE" => Self::Mode,
            "RST_SENT" => Self::SentRST,
            "RST_RCVD" => Self::RcvdRST,
            "GRIDSQUARE" => Self::GridSquare,
            "STATE" => Self::PrimaryAdminSubdiv,
            "STX" => Self::SentSerial,
            "SRX" => Self::RcvdSerial,
            "COMMENT" => Self::Comment,
            _ => Self::Other(field_name.into()),
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct LogRecord(HashMap<String, String>);

impl LogRecord {
    pub fn new() -> Self {
        LogRecord(HashMap::new())
    }

    pub fn insert_field(&mut self, ty: FieldType, val: &str) -> &mut Self {
        self.0.insert(ty.to_string(), val.to_string());
        self
    }

    pub fn insert_timestamp(&mut self, ts: Timestamp) -> &mut Self {
        self.0
            .insert(FieldType::Timestamp.to_string(), ts.to_string());
        self
    }

    pub fn get_field(&self, ty: FieldType) -> Result<String, LogError> {
        match self.0.get(&ty.to_string()) {
            Some(val) => Ok(val.to_string()),
            None => {
                return Err(LogError {
                    message: "Could not get keyvalue pair from LogRecord".to_string(),
                    offender: ty.to_string(),
                });
            }
        }
    }

    pub fn into_iter(&self) -> std::collections::hash_map::IntoIter<String, String> {
        self.0.clone().into_iter()
    }
}

impl Display for LogRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.0 {
            writeln!(f, "{} {}", key, value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Encode, Decode)]
pub struct LogHeader {
    version: String,
    op_call: String,
    comment: String,
}

#[derive(Debug)]
pub struct Log {
    db: Db,
    idx: usize,
}

impl Log {
    pub fn new(db: Db) -> Result<Self, LogError> {
        match db.get(b"MAGIC") {
            Ok(db_value) => {
                match db_value {
                    Some(val) => {
                        if val.to_ascii_uppercase().as_slice() == VEELOG_MAGIC {
                            // we can presume that this is a safe existing database. continue as normal.
                            Ok(Log { db, idx: 0 })
                        } else {
                            // not our magic. error
                            Err(LogError {
                                message: "MAGIC exists and does not match".to_string(),
                                offender: format!("{:?}", val),
                            })
                        }
                    }
                    None => {
                        if db.is_empty() {
                            // empty database. make a new one
                            let log = Log { db, idx: 0 };
                            log.init_db()?;
                            Ok(log)
                        } else {
                            // this db is NOT empty and not ours. error
                            Err(LogError {
                                message: "Non-empty unknown database".to_string(),
                                offender: format!("somewhere"),
                            })
                        }
                    }
                }
            }
            // unknown error in opening database
            Err(e) => Err(LogError {
                message: "Could not get MAGIC".to_string(),
                offender: e.to_string(),
            }),
        }
    }

    fn init_db(&self) -> Result<(), LogError> {
        match self.db.insert(b"MAGIC", VEELOG_MAGIC) {
            Ok(_) => {
                // magic inserted fine, insert info string
                match self.db.insert(b"INFO", "Database generated by veelog. Visit https://github.com/hf-ikea/veelog for more information.") {
                    Ok(_) => {
                        // fresh new db for hot qsos
                        Ok(())
                    },
                    // unknown error in magic insertion
                    Err(e) => Err(LogError {
                        message: "Could insert magic".to_string(),
                        offender: e.to_string(),
                    }),
                }
            }
            // unknown error in info string insertion
            Err(e) => Err(LogError {
                message: "Could insert db info string".to_string(),
                offender: e.to_string(),
            }),
        }
    }

    pub fn get_record(&self, idx: usize) -> Result<LogRecord, LogError> {
        match self.db.get(idx.to_le_bytes()) {
            Ok(val) => match val {
                Some(enc) => {
                    let dec =
                        decode_from_slice::<LogRecord, Configuration>(&enc, config::standard());
                    match dec {
                        Ok(v) => Ok(v.0),
                        Err(e) => Err(LogError {
                            message: "Could not deserialize record".to_string(),
                            offender: e.to_string(),
                        }),
                    }
                }
                None => Err(LogError {
                    message: "Record does not exist".to_string(),
                    offender: idx.to_string(),
                }),
            },
            Err(e) => Err(LogError {
                message: e.to_string(),
                offender: idx.to_string(),
            }),
        }
    }

    pub fn insert_record(&mut self, record: LogRecord) -> Result<(), LogError> {
        self.modify_record(self.idx, record)?;
        self.idx += 1;
        Ok(())
    }

    pub fn modify_record(&self, idx: usize, record: LogRecord) -> Result<(), LogError> {
        let enc = data::Log::encode_record(record)?;

        match self.db.insert(idx.to_le_bytes(), enc) {
            Ok(_) => Ok(()),
            Err(_) => todo!(), // some error in inserting to the db, this is not caused by dupes
        }
    }

    fn encode_record(record: LogRecord) -> Result<Vec<u8>, LogError> {
        if record.0.is_empty() {
            return Err(LogError {
                message: "Empty record".to_string(),
                offender: record.to_string(),
            });
        }

        match encode_to_vec(&record, config::standard()) {
            Ok(val) => Ok(val),
            Err(_) => todo!(), // no clue what to even do if it fails. give up?
        }
    }

    pub fn import_adif(&mut self, adif: ADIFFile) -> Result<(), LogError> {
        for adif_record in adif.body {
            let mut log_record = LogRecord::new();
            let mut date: Option<Date> = None;
            let mut time: Option<Time> = None;
            for (field_name, value) in adif_record {
                let val = &value.extract_value()?;
                match field_name.as_str() {
                    "QSO_DATE" => {
                        match strtime::parse("%Y%m%d", val) {
                            Ok(t) => match t.to_date() {
                                Ok(v) => date = Some(v),
                                Err(e) => {
                                    return Err(LogError {
                                        message: format!("Could not parse QSO_DATE {}", val),
                                        offender: e.to_string(),
                                    });
                                }
                            },
                            Err(e) => {
                                return Err(LogError {
                                    message: format!("Could not parse QSO_DATE {}", val),
                                    offender: e.to_string(),
                                });
                            }
                        }
                    }
                    "TIME_ON" => {
                        match strtime::parse("%H%M%S", val) {
                            Ok(t) => match t.to_time() {
                                Ok(v) => time = Some(v),
                                Err(e) => {
                                    return Err(LogError {
                                        message: format!("Could not parse TIME_ON {}", val),
                                        offender: e.to_string(),
                                    });
                                }
                            },
                            Err(e) => {
                                return Err(LogError {
                                    message: format!("Could not parse TIME_ON {}", val),
                                    offender: e.to_string(),
                                });
                            }
                        }
                    }
                    "STATION_CALLSIGN" => continue,
                    "MY_GRIDSQUARE" => continue,
                    "BAND" => continue,
                    "TIME_OFF" => continue,
                    "QSO_DATE_OFF" => continue,
                    "TX_PWR" => continue,
                    "SUBMODE" => continue,
                    _ => {
                        log_record.insert_field(FieldType::from_adif_field(&field_name), val);
                    }
                }
            }
            if let Some(d) = date {
                if let Some(t) = time {
                    let ts = d.to_datetime(t).to_zoned(TimeZone::UTC).unwrap().timestamp();
                    log_record.insert_timestamp(ts);
                }
            } else {
                return Err(LogError {
                    message: "ADIF record had no date and/or time fields".to_string(),
                    offender: format!(""),
                });
            }
            self.insert_record(log_record)?;
        }
        Ok(())
    }

    pub fn get_len(&self) -> usize {
        self.idx
    }
}