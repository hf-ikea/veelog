use anyhow::{bail, Result};

pub fn pretty_gridsquare(grid: &String) -> Result<String> {
    if !grid.is_ascii() {
        bail!("Gridsquare is not ASCII: {}", grid)
    }
    match grid.len() {
        4 => {
            Ok(grid.to_ascii_uppercase())
        }
        6 => {
            Ok(grid[..4].to_uppercase() + &grid[4..].to_lowercase())
        }
        _ => bail!("GRIDSQUARE is of invalid length {}: {}", grid.len(), grid)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::pretty_gridsquare;

    #[test]
    pub fn test_prettify_grid() {
        let grid = "aA00aA".to_string();
        assert_eq!("AA00aa".to_string(), pretty_gridsquare(&grid).unwrap());
        let grid = "aa00AA".to_string();
        assert_eq!("AA00aa".to_string(), pretty_gridsquare(&grid).unwrap());
        let grid = "aa00".to_string();
        assert_eq!("AA00".to_string(), pretty_gridsquare(&grid).unwrap());
    }
}
