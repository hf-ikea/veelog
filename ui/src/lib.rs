use std::fs;

#[cfg(test)]
mod tests {
    use std::{cmp::max, fs};

    use adif::parse;

    #[test]
    fn print_adif() {
        let data: String = fs::read_to_string("../testlog.adi").unwrap();
        let adif = parse::parse_adif(&data);

        // let mut max_field_len: usize = 0;
        // for (field_name, value) in adif.header.clone() {
        //     max_field_len = std::cmp::max(max_field_len, field_name.len());
        //     max_field_len = std::cmp::max(max_field_len, value.to_string().len());
        // }

        let mut field_print = "|".to_string();
        let mut val_print = "|".to_string();

        for (field_name, val) in adif.header.clone() {
            let len = max(val.to_string().len(), field_name.len());
            field_print.push_str(&format!(" {:<len$} |", field_name));
            val_print.push_str(&format!(" {:<len$} |", val.to_string()));
        }
        println!("{}\n{}", field_print, val_print);
    }
}
