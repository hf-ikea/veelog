use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{message:?}. Offending value: {offender:?}")]
    ADIFSerializeError { message: String, offender: String },
    #[error("Could not parse {field_name:?}. Offending value: {field_value:?}. More info: {err:?}")]
    FieldParseError {
        field_name: String,
        field_value: String,
        err: String,
    },
    #[error("Key {0:?} does not exist in database.")]
    DatabaseGetError(String),
}


pub fn prettyvalidate_gridsquare(grid: &String) -> Result<String> {
    if !grid.is_ascii() {
        anyhow::bail!("Gridsquare is not ASCII: {}", grid)
    }
    match grid.len() {
        4 => Ok(grid.to_ascii_uppercase()),
        6 => Ok(grid[..4].to_uppercase() + &grid[4..].to_lowercase()),
        _ => anyhow::bail!("GRIDSQUARE is of invalid length {}: {}", grid.len(), grid),
    }
}

#[cfg(test)]
mod tests {
    use crate::prettyvalidate_gridsquare;

    #[test]
    pub fn test_prettify_grid() {
        let grid = "aA00aA".to_string();
        assert_eq!(
            "AA00aa".to_string(),
            prettyvalidate_gridsquare(&grid).unwrap()
        );
        let grid = "aa00AA".to_string();
        assert_eq!(
            "AA00aa".to_string(),
            prettyvalidate_gridsquare(&grid).unwrap()
        );
        let grid = "aa00".to_string();
        assert_eq!(
            "AA00".to_string(),
            prettyvalidate_gridsquare(&grid).unwrap()
        );
    }
}
