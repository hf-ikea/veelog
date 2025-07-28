use chrono::Local;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ADIFType {
    Str(String),
    Bool(bool),
    Num(f64),
}

impl ADIFType {
    pub fn serialize(&self, field_name: &str) -> Result<String> {
        let value = match self {
            ADIFType::Str(val) => val.to_string(),
            ADIFType::Bool(_) => todo!(),
            ADIFType::Num(_) => todo!(),
        };
        Ok(format!(
            "<{}:{}{}>{}",
            field_name.to_uppercase().replace(" ", "_"),
            value.len(),
            String::new(),
            value
        ))
    }

    pub fn extract_value(&self) -> Result<String> {
        match self {
            ADIFType::Str(v) => Ok(v.to_string()),
            _ => {
                Err(util::Error::ADIFSerializeError {
                    message: "Cannot handle ADIF record with type".to_string(),
                    offender: self.to_string(),
                })?
            }
        }
    }
}

impl std::fmt::Display for ADIFType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADIFType::Str(v) => write!(f, "{}", v),
            ADIFType::Bool(v) => write!(f, "{}", v),
            ADIFType::Num(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ADIFHeader(pub Vec<(String, ADIFType)>);

impl ADIFHeader {
    pub fn serialize(&self) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!(
            "Exported from veelog on {}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        let header = self
            .0
            .iter()
            .map(|(key, val)| val.serialize(key))
            .collect::<Result<Vec<String>>>()?
            .join("\n");
        out.push_str(&header);
        out.push('\n');
        out.push_str("<EOH>");
        Ok(out)
    }
}

impl IntoIterator for ADIFHeader {
    type Item = (String, ADIFType);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ADIFRecord(pub Vec<(String, ADIFType)>);

impl ADIFRecord {
    pub fn serialize(&self) -> Result<String> {
        let mut out = self
            .0
            .iter()
            .map(|(key, val)| val.serialize(key))
            .collect::<Result<Vec<String>>>()?
            .join("");
        out.push_str("<EOR>");
        Ok(out)
    }
}

impl IntoIterator for ADIFRecord {
    type Item = (String, ADIFType);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ADIFFile {
    pub header: ADIFHeader,
    pub body: Vec<ADIFRecord>,
}

impl ADIFFile {
    pub fn new(header: ADIFHeader, body: Vec<ADIFRecord>) -> Self {
        ADIFFile { header, body }
    }

    pub fn serialize(&self) -> Result<String> {
        let mut output = self.header.serialize()?;
        output.push('\n');
        let records = self
            .body
            .iter()
            .map(|r| r.serialize())
            .collect::<Result<Vec<String>>>()?
            .join("\n");
        output.push_str(&records);
        Ok(output)
    }
}
