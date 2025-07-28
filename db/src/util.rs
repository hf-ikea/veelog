use anyhow::{bail, Result};

pub fn pretty_gridsquare(grid: &String) -> Result<String> {
    let mut grid = grid.to_string();
    if !grid.is_ascii() {
        bail!("GRIDSQUARE is not ASCII: {}", grid)
    }
    match grid.len() {
        4 => {
            Ok(grid.to_ascii_uppercase())
        }
        6 => {
            let suffix = &grid.clone()[4..];
            grid = grid[..4].to_uppercase();
            grid += suffix;
            dbg!(&grid);
            Ok(grid)
        }
        _ => bail!("GRIDSQUARE is of invalid length {}: {}", grid.len(), grid)
    }
}