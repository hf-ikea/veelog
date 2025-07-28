use adif::data::ADIFFile;
use util::Error;
use crate::{VEELOG_MAGIC, util::pretty_gridsquare};

use bincode::{
    Decode, Encode,
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
};
use indexmap::IndexMap;
use jiff::{
    Timestamp,
    civil::{Date, Time},
    fmt::strtime,
    tz::TimeZone,
};
use sled::{Db, IVec};
use std::{fmt::Display, ops::Deref};
use anyhow::{Result, bail};

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
    Frequency, // freq in MHz
    Mode,
    SentRST,
    RcvdRST,
    GridSquare,
    PrimaryAdminSubdiv,
    SentSerial,
    RcvdSerial,
    DXCC,
    CQZ,
    ITUZ,
    POTARef,
    Comment,
    Name,
    QTH,
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
            "DXCC" => Self::DXCC,
            "CQZ" => Self::CQZ,
            "ITUZ" => Self::ITUZ,
            "POTA_REF" => Self::POTARef,
            "COMMENT" => Self::Comment,
            "NAME" => Self::Name,
            "QTH" => Self::QTH,
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
pub struct LogRecord {
    #[bincode(with_serde)]
    map: IndexMap<String, String>,
}

impl LogRecord {
    pub fn new() -> Self {
        LogRecord {
            map: IndexMap::new(),
        }
    }

    pub fn insert_field(&mut self, ty: FieldType, val: &str) -> &mut Self {
        self.map.insert(ty.to_string(), val.to_string());
        self
    }

    pub fn insert_timestamp(&mut self, ts: Timestamp) -> &mut Self {
        self.map
            .insert(FieldType::Timestamp.to_string(), ts.to_string());
        self
    }

    pub fn get_field(&self, ty: FieldType) -> Result<String, LogError> {
        match self.map.get(&ty.to_string()) {
            Some(val) => Ok(val.to_string()),
            None => {
                return Err(LogError {
                    message: "Could not get keyvalue pair from LogRecord".to_string(),
                    offender: ty.to_string(),
                });
            }
        }
    }
}

impl Deref for LogRecord {
    type Target = IndexMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Display for LogRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.map {
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

impl LogHeader {
    pub fn new(op_call: &str, comment: &str) -> Self {
        Self {
            version: "0.1.0".to_string(),
            op_call: op_call.to_string(),
            comment: comment.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Log {
    db: Db,
}

impl Log {
    /// Creates a new Log object with a passed in sled Db that must be already intialized
    pub fn new(db: Db) -> Result<Self> {
        let log = Self { db };
        let db_value = log.get_key(b"MAGIC")?;
        match db_value {
            Some(val) => {
                if val.to_ascii_uppercase().as_slice() == VEELOG_MAGIC {
                    // we can presume that this is a safe existing database. continue as normal.
                    Ok(log)
                } else {
                    // not our magic. error
                    bail!("MAGIC exists and does not match: {:?}", val)
                }
            }
            None => bail!("Uninitalized database. Use Log::new_init() instead of Log::new()"),
        }
    }

    /// Creates a new initalized Log. Should be used when the db is fresh
    pub fn new_init(db: Db, header: LogHeader) -> Result<Self> {
        if db.is_empty() {
            // empty database. make a new one
            let log = Self { db };
            log.init_db(header)?;
            Ok(log)
        } else {
            // this db is NOT empty. error
            bail!("Non-empty database used in Log::new_init()")
        }
    }

    fn init_db(&self, header: LogHeader) -> Result<()> {
        self.set_key(b"MAGIC", VEELOG_MAGIC)?;
        self.set_key(b"INFO", "Database generated by veelog. Visit https://github.com/hf-ikea/veelog for more information.")?;
        self.set_idx(0)?;
        self.set_key(b"HEADER", Self::encode_record(header)?)
    }

    fn get_key(&self, key: &[u8]) -> Result<Option<IVec>> {
        match self.db.get(key) {
            Ok(db_value) => Ok(db_value),
            // unknown error in opening database
            Err(e) => bail!(e),
        }
    }

    pub fn get_idx(&self) -> Result<usize> {
        match self.get_key(b"INDEX")? {
            Some(v) => match v.to_vec().try_into() {
                Ok(v) => Ok(usize::from_le_bytes(v)),
                Err(e) => bail!("Invalid INDEX value. {:?}", e),
            },
            None => bail!(Error::DatabaseGetError("INDEX".to_string())),
        }
    }

    fn set_idx(&self, idx: usize) -> Result<()> {
        self.set_key(b"INDEX", &idx.to_le_bytes())
    }

    fn set_key<T: Into<IVec>>(&self, key: &[u8], val: T) -> Result<()> {
        match self.db.insert(key, val) {
            Ok(_) => Ok(()),
            // unknown error in key insertion
            Err(e) => bail!(e),
        }
    }

    pub fn get_header(&self) -> Result<LogHeader> {
        match self.get_key(b"HEADER")? {
            Some(v) => Self::decode_record(&v),
            None => bail!("HEADER does not exist in db"),
        }
    }

    pub fn get_record(&self, idx: usize) -> Result<LogRecord> {
        match self.db.get(idx.to_le_bytes()) {
            Ok(val) => match val {
                Some(enc) => Self::decode_record(&enc),
                None => bail!(Error::DatabaseGetError(idx.to_string())),
            },
            Err(e) => bail!(LogError {
                message: e.to_string(),
                offender: idx.to_string(),
            }),
        }
    }

    pub fn insert_record(&mut self, record: LogRecord) -> Result<()> {
        let idx = self.get_idx()?;
        self.modify_record(idx, record)?;
        self.set_idx(idx + 1)?;
        Ok(())
    }

    pub fn modify_record(&self, idx: usize, record: LogRecord) -> Result<()> {
        let enc = Self::encode_record(record)?;

        match self.db.insert(idx.to_le_bytes(), enc) {
            Ok(_) => Ok(()),
            Err(_) => todo!(), // some error in inserting to the db, this is not caused by dupes
        }
    }

    fn encode_record(record: impl Encode) -> Result<Vec<u8>> {
        match encode_to_vec(&record, config::standard()) {
            Ok(val) => Ok(val),
            Err(_) => todo!(), // no clue what to even do if it fails. give up?
        }
    }

    fn decode_record<T: bincode::de::Decode<()>>(enc: &[u8]) -> Result<T> {
        match decode_from_slice::<T, Configuration>(enc, config::standard()) {
            Ok(val) => Ok(val.0),
            Err(_) => todo!(), // no clue what to even do if it fails. give up?
        }
    }

    pub fn import_adif(&mut self, adif: ADIFFile) -> Result<()> {
        for adif_record in adif.body {
            let mut log_record = LogRecord::new();
            let mut date: Option<Date> = None;
            let mut time: Option<Time> = None;
            for (field_name, value) in adif_record {
                let val = &value.extract_value()?;
                let field_name = field_name.as_str();
                match field_name.get(..3) {
                    Some("MY_") => continue,
                    Some("SIG") => continue,
                    Some("QSL") => continue,
                    _ => match field_name {
                        "STATION_CALLSIGN" => continue,
                        "OPERATOR" => continue,
                        "BAND" => continue,
                        "TIME_OFF" => continue,
                        "QSO_DATE_OFF" => continue,
                        "TX_PWR" => continue,
                        "SUBMODE" => continue,
                        "FREQ" => {
                            log_record.insert_field(
                                FieldType::from_adif_field(&field_name),
                                val.trim_matches('0'),
                            );
                        }
                        "GRIDSQUARE" => {
                            log_record.insert_field(
                                FieldType::from_adif_field(&field_name),
                                &pretty_gridsquare(val)?,
                            );
                        }
                        "QSO_DATE" => match strtime::parse("%Y%m%d", val) {
                            Ok(t) => match t.to_date() {
                                Ok(v) => date = Some(v),
                                Err(e) => {
                                    bail!(util::Error::FieldParseError {
                                        field_name: field_name.to_string(),
                                        field_value: val.to_string(),
                                        err: e.to_string(),
                                    });
                                }
                            },
                            Err(e) => {
                                bail!(util::Error::FieldParseError {
                                    field_name: field_name.to_string(),
                                    field_value: val.to_string(),
                                    err: e.to_string(),
                                });
                            }
                        },
                        "TIME_ON" => match strtime::parse("%H%M%S", val) {
                            Ok(t) => match t.to_time() {
                                Ok(v) => time = Some(v),
                                Err(e) => {
                                    bail!(util::Error::FieldParseError {
                                        field_name: field_name.to_string(),
                                        field_value: val.to_string(),
                                        err: e.to_string(),
                                    });
                                }
                            },
                            Err(e) => {
                                bail!(util::Error::FieldParseError {
                                    field_name: field_name.to_string(),
                                    field_value: val.to_string(),
                                    err: e.to_string(),
                                });
                            }
                        },
                        _ => {
                            log_record.insert_field(FieldType::from_adif_field(&field_name), val);
                        }
                    },
                }
            }
            if let Some(d) = date {
                if let Some(t) = time {
                    let ts = d
                        .to_datetime(t)
                        .to_zoned(TimeZone::UTC)
                        .unwrap()
                        .timestamp();
                    log_record.insert_timestamp(ts);
                }
            } else {
                bail!("ADIF record had no date and/or time fields");
            }
            self.insert_record(log_record)?;
        }
        Ok(())
    }
}
