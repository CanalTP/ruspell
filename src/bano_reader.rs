pub fn populate_dict_from_bano_file(file: &str, ispell: &mut ::ispell_wrapper::SpellCheck) {
    println!("Reading street and city names from {}", file);

    ispell.add_word("L'Haÿ-les-Roses").unwrap();
}

