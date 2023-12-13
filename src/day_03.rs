use num::ToPrimitive;
use std::error::Error;
use std::io::{BufRead, BufReader, Cursor};
use std::{env, fs};

#[derive(Clone, PartialEq)]
enum CellType {
    Period,
    Symbol,
    Digit,
    Gear,
}

#[derive(Clone)]
struct Cell {
    cell_value: usize,
    cell_type: CellType,
}

impl Cell {
    fn new(cell_value: usize, cell_type: CellType) -> Self {
        Cell {
            cell_value,
            cell_type,
        }
    }

    fn is_digit(&self) -> bool {
        self.cell_type == CellType::Digit
    }

    fn is_symbol_or_gear(&self) -> bool {
        self.cell_type == CellType::Symbol || self.cell_type == CellType::Gear
    }

    fn is_gear(&self) -> bool {
        self.cell_type == CellType::Gear
    }
}

pub fn totals() -> Result<(usize, usize), Box<dyn Error>> {
    let file_path = format!(
        "{}/data/input_day_03.txt",
        env::current_dir().unwrap().display()
    );
    Ok((
        sum_part_numbers(fs::read_to_string(&file_path)?)?,
        sum_gear_ratios(fs::read_to_string(&file_path)?)?,
    ))
}

fn sum_part_numbers(input: String) -> Result<usize, Box<dyn Error>> {
    let schem = create_padded_schematic(input)?;
    let mut sum: usize = 0;

    for (row_index, row) in schem.iter().enumerate() {
        // Skip first and last rows as these are just for padding.
        if row_index == 0 || row_index == (schem.len() - 1) {
            continue;
        }

        let mut digits: Vec<usize> = vec![];

        for (cell_index, cell) in row.iter().enumerate() {
            if cell.is_digit() {
                digits.push(cell.cell_value);
                // The last cell will always be padding so skip to next.
                continue;
            }

            // Check if there are any digits to handle, otherwise skip.
            if digits.is_empty() {
                continue;
            }

            let mut is_part: bool = false;
            let scan_width: usize = digits.len();

            // Scan around the cell.
            for i in 0..scan_width {
                // Scan above.
                if schem[row_index - 1][(cell_index - scan_width) + i].is_symbol_or_gear() {
                    is_part = true;
                }

                // Scan below.
                if schem[row_index + 1][(cell_index - scan_width) + i].is_symbol_or_gear() {
                    is_part = true;
                }

                // Scan left.
                if i == 0 {
                    if schem[row_index][cell_index - (scan_width + 1)].is_symbol_or_gear() {
                        is_part = true;
                    }

                    if schem[row_index - 1][cell_index - (scan_width + 1)].is_symbol_or_gear() {
                        is_part = true;
                    }

                    if schem[row_index + 1][cell_index - (scan_width + 1)].is_symbol_or_gear() {
                        is_part = true;
                    }
                }

                // Scan right.
                if i == (scan_width - 1) {
                    if schem[row_index][cell_index].is_symbol_or_gear() {
                        is_part = true;
                    }

                    if schem[row_index - 1][cell_index].is_symbol_or_gear() {
                        is_part = true;
                    }

                    if schem[row_index + 1][cell_index].is_symbol_or_gear() {
                        is_part = true;
                    }
                }
            }

            if is_part {
                // We now need to convert the vec of usize into strings, concatenate them, and
                // convert back to usize :facepalm: Must be a nicer way than this...
                let string_vals: Vec<String> = digits.iter().map(|&n| n.to_string()).collect();
                let concat_string: String = string_vals.join("");
                sum += concat_string.parse::<usize>()?;
            }

            digits.clear();
        }
    }

    Ok(sum)
}

fn sum_gear_ratios(input: String) -> Result<usize, Box<dyn Error>> {
    let schem = create_padded_schematic(input)?;
    let mut part_num_sum: Vec<usize> = vec![];

    for (row_index, row) in schem.iter().enumerate() {
        // Skip first and last rows as these are just for padding.
        if row_index == 0 || row_index == (schem.len() - 1) {
            continue;
        }

        let mut part_nums: Vec<usize> = vec![];

        for (cell_index, cell) in row.iter().enumerate() {
            // Skip first cell as this is just padding.
            if cell_index == 0 {
                continue;
            }

            if !cell.is_gear() {
                continue;
            }

            let mut part_num: Vec<usize> = vec![];

            for i in 0..2 {
                // Scan above.
                if schem[row_index - 1][(cell_index - 1) + i].is_digit() {
                    if let Some(p) = scan_part_num(&schem, row_index - 1, (cell_index - 1) + i) {
                        part_num.push(p)
                    }
                }

                // Scan below.
                if schem[row_index + 1][(cell_index - i) + i].is_digit() {
                    if let Some(p) = scan_part_num(&schem, row_index + 1, (cell_index - 1) + i) {
                        part_num.push(p)
                    }
                }
            }

            // Scan left.
            if schem[row_index][cell_index - 1].is_digit() {
                if let Some(p) = scan_part_num(&schem, row_index, cell_index - 1) {
                    part_num.push(p)
                }
            }

            // Scan right.
            if schem[row_index][cell_index + 1].is_digit() {
                if let Some(p) = scan_part_num(&schem, row_index, cell_index + 1) {
                    part_num.push(p)
                }
            }

            if !part_num.is_empty() {
                // We now need to convert the vec of usize into strings, concatenate them, and
                // convert back to usize :facepalm: Must be a nicer way than this...
                let string_vals: Vec<String> = part_nums.iter().map(|&n| n.to_string()).collect();
                let concat_string: String = string_vals.join("");
                part_nums.push(concat_string.parse::<usize>()?);
            }
        }

        if !part_nums.is_empty() {
            part_num_sum.push(part_nums.iter().cloned().fold(0, |acc, x| acc * x));
        }
    }

    Ok(part_num_sum.iter().sum())
}

fn scan_part_num(schem: &[Vec<Cell>], col: usize, row: usize) -> Option<usize> {
    let mut part_num: Vec<usize> = vec![];
    let mut pos: usize = col;

    loop {
        part_num.insert(0, schem[row][pos].cell_value);
        if pos == 0 {
            break;
        }
        pos -= 1;
        if !schem[row][pos].is_digit() {
            break;
        }
    }

    Some(part_num.iter().sum())
}

fn create_padded_schematic(input: String) -> Result<Vec<Vec<Cell>>, Box<dyn Error>> {
    let cursor = Cursor::new(input);
    let reader = BufReader::new(cursor);

    let mut schematic: Vec<Vec<Cell>> = Vec::new();
    let mut line_len: usize = 0;

    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap_or_default();

        // Skip empty lines.
        if line.is_empty() {
            continue;
        }

        // We only need to set the length once.
        if i == 0 {
            line_len = line.len().to_usize().unwrap();
        }

        let mut row: Vec<Cell> = Vec::new();
        for char in line.chars() {
            let (cell_value, cell_type) = match char {
                '.' => (0, CellType::Period),
                '0'..='9' => {
                    let value = char.to_digit(10).ok_or("invalid digit")?;
                    (value.to_usize()?, CellType::Digit)
                }
                '*' => (0, CellType::Gear),
                _ => (0, CellType::Symbol),
            };

            let cell = Box::new(Cell::new(cell_value, cell_type));
            row.push(*cell);
        }

        // Pad the left and right to minimise out of bound checking.
        row.insert(0, Cell::new(0, CellType::Period));
        row.push(Cell::new(0, CellType::Period));

        schematic.push(row);
    }

    // Pad top and bottom to minimise out of bound checking.
    schematic.insert(
        0,
        (0..(line_len + 2).to_usize().unwrap_or_default())
            .map(|_| Cell::new(0, CellType::Period))
            .collect(),
    );
    schematic.push(
        (0..(line_len + 2).to_usize().unwrap_or_default())
            .map(|_| Cell::new(0, CellType::Period))
            .collect(),
    );

    Ok(schematic)
}

#[cfg(test)]
mod tests {
    use super::{sum_gear_ratios, sum_part_numbers};

    fn test_sum_part_numbers_case(input: &str, want: usize) -> Result<(), String> {
        match sum_part_numbers(String::from(input)) {
            Ok(got) => {
                if got != want {
                    Err(format!("want {}, got {}", want, got))
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(format!("{}", err.to_string())),
        }
    }

    #[test]
    fn test_sum_part_numbers() -> Result<(), String> {
        [
            (
                r#"*100......
.........."#,
                100,
            ),
            (
                r#"200*......
.........."#,
                200,
            ),
            (
                r#"300.......
...*......"#,
                300,
            ),
            (
                r#".400......
*........."#,
                400,
            ),
            (
                r#"...*......
500......."#,
                500,
            ),
            (
                r#"*.........
.600......"#,
                600,
            ),
            (
                r#"..*.......
.700......"#,
                700,
            ),
            (
                r#".800......
..*......."#,
                800,
            ),
            (
                r#".900......
...*......"#,
                900,
            ),
            (
                r#".10.......
.*........"#,
                10,
            ),
            (
                r#"..12......
.........*"#,
                0,
            ),
            (
                r#"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598.."#,
                4361,
            ),
            (r#""#, 0),
            (r#"."#, 0),
            (r#".........."#, 0),
        ]
        .iter()
        .try_for_each(|(input, want)| test_sum_part_numbers_case(*input, *want))?;

        Ok(())
    }

    fn test_sum_gear_ratios_case(input: &str, want: usize) -> Result<(), String> {
        match sum_gear_ratios(String::from(input)) {
            Ok(got) => {
                if got != want {
                    Err(format!("want {}, got {}", want, got))
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(format!("{}", err.to_string())),
        }
    }

    #[test]
    fn test_sum_gear_ratios() -> Result<(), String> {
        [
            (
                r#"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598.."#,
                467835,
            ),
            (r#""#, 0),
            (r#"."#, 0),
            (r#".........."#, 0),
        ]
        .iter()
        .try_for_each(|(input, want)| test_sum_gear_ratios_case(*input, *want))?;

        Ok(())
    }
}
