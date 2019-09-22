use std::io;

fn main() {
    // Set up Booleans for meter length classification
    let mut long_meter = false;
    let mut short_meter = false;

    // Set up Booleans for other clues
    let mut found_initial_clues = false;
    let mut initial_agar = false;
    let mut initial_chun = false;
    let mut initial_kasi = false;
    let mut initial_yaki = false;

    // Create integer variable to count letters
    let mut total_letters: u32 = 0;

    // Set up variables for checking syllable length
    let mut long_first_syllable_markers: u32 = 0;
    let mut long_first_syllable_locs = String::new();
    let mut short_first_syllable_markers: u32 = 0;
    let mut short_first_syllable_locs = String::new();
    let mut new_short_first_syllable_markers: u32 = 0;
    let mut long_second_syllable_markers: u32 = 0;
    let mut long_second_syllable_locs = String::new();
    let mut new_long_second_syllable_markers: u32 = 0;
    let mut short_second_syllable_markers: u32 = 0;
    let mut short_second_syllable_locs = String::new();

    // Prompt input
    println!("Enter ten consecutive hemistichs:");

    // Set up hemistich counter
    let mut hem_id = 1;

    //
    // Data-gathering loop
    //

    for _ in 0..10 {
        // io
        let mut hemistich = String::new();
        io::stdin()
            .read_line(&mut hemistich)
            .expect("Failed to read line…");

        // Reconstruct the hemistich as a vector of chars; create a version without spaces
        let hem_recon: Vec<char> = reconstruct_hemistich(hemistich);
        let mut hem_recon_nospace = hem_recon.clone();
        hem_recon_nospace.retain(|x| *x != ' ');

        // Count chars (excluding spaces), and add to the total
        let hem_letter_count = hem_recon_nospace.len();
        total_letters += hem_letter_count as u32;

        // Check for long first syllable
        let long_first_syllable_result = long_first_syllable(hem_recon.clone());
        if long_first_syllable_result > 0 {
            long_first_syllable_markers += long_first_syllable_result;
            long_first_syllable_locs.push_str(&hem_id.to_string());
            long_first_syllable_locs.push_str(", ");
        }

        // Check for short first syllable
        let short_first_syllable_result = short_first_syllable(hem_recon.clone());
        if short_first_syllable_result > 0 {
            short_first_syllable_markers += short_first_syllable_result;
            short_first_syllable_locs.push_str(&hem_id.to_string());
            short_first_syllable_locs.push_str(", ");
        }

        // Check for long second syllable
        let long_second_syllable_result =
            long_second_syllable(hem_recon.clone(), hem_recon_nospace.clone());
        if long_second_syllable_result > 0 {
            long_second_syllable_markers += long_second_syllable_result;
            long_second_syllable_locs.push_str(&hem_id.to_string());
            long_second_syllable_locs.push_str(", ");
        }

        // Check for short second syllable
        let short_second_syllable_result =
            short_second_syllable(hem_recon.clone(), hem_recon_nospace.clone());
        if short_second_syllable_result > 0 {
            short_second_syllable_markers += short_second_syllable_result;
            short_second_syllable_locs.push_str(&hem_id.to_string());
            short_second_syllable_locs.push_str(", ");
        }

        // Check for hemistich-initial clues
        // 'a' means initial "agar" followed by a consonant (after a space)
        // 'c' means initial "chunīn" or "chunān"
        // 'k' means initial "kasī" followed by a consonant (after a space)
        // 'y' means initial "yakī" followed by a consonant (after a space)
        let initial_clues_result = initial_clues(hem_recon.clone());
        if !initial_clues_result.is_empty() {
            found_initial_clues = true;
            for c in initial_clues_result.iter() {
                match *c {
                    'a' => {
                        initial_agar = true;
                        new_long_second_syllable_markers += 1;
                    }
                    'c' => {
                        initial_chun = true;
                        new_long_second_syllable_markers += 1;
                    }
                    'k' => {
                        initial_kasi = true;
                        new_short_first_syllable_markers += 1;
                        new_long_second_syllable_markers += 1;
                    }
                    'y' => {
                        initial_yaki = true;
                        new_short_first_syllable_markers += 1;
                        new_long_second_syllable_markers += 1;
                    }
                    _ => {}
                }
            }
        }

        // Add to counter
        hem_id += 1;
    }

    //
    // Results
    //

    // Calculate average letters per hemistich
    let total_letters_float = f64::from(total_letters);
    let avg_letters: f64 = total_letters_float / 10.0;

    // Report assessment of meter length
    println!("*** Meter length ***");
    println!("Average letters per hemistich: {:.1}", avg_letters);
    if avg_letters >= 23.0 {
        long_meter = true;
        println!("The meter appears to be long (muṡamman).");
    } else if avg_letters >= 21.0 {
        println!("It is not obvious whether the meter is long or short.");
        println!("(In this grey area, the answer is usually long.)");
    } else {
        short_meter = true;
        println!("The meter appears to be short (musaddas; or mutaqārib muṡamman).");
    }

    // Report assessment of first syllable length
    let (mut long_first, mut short_first) = first_syllable_assessment(
        long_first_syllable_markers,
        long_first_syllable_locs,
        short_first_syllable_markers,
        short_first_syllable_locs,
    );

    // Report assessment of second syllable length
    let (mut long_second, mut short_second) = second_syllable_assessment(
        long_second_syllable_markers,
        long_second_syllable_locs,
        short_second_syllable_markers,
        short_second_syllable_locs,
    );

    // Address other hemistich-initial clues, if any
    if found_initial_clues {
        // Report clues
        println!("*** Other hemistich-initial clues ***");
        if initial_agar {
            println!("Found initial 'agar' followed by a consonant.");
            println!("This suggests a short first, and long second syllable.");
        }
        if initial_chun {
            println!("Found initial 'chunīn' or 'chunān'.");
            println!("This suggests a short first, and long second syllable.");
        }
        if initial_kasi {
            println!("Found initial 'kasī' followed by a consonant.");
            println!("This suggests a short first, and long second syllable.");
        }
        if initial_yaki {
            println!("Found initial 'yakī' followed by a consonant.");
            println!("This suggests a short first, and long second syllable.");
        }

        // Add new syllable length markers
        short_first_syllable_markers += new_short_first_syllable_markers;
        long_second_syllable_markers += new_long_second_syllable_markers;

        // Reassess first syllable length
        if long_first_syllable_markers > 0 && short_first_syllable_markers > 0 {
            println!("There are now contradictory indications of the first syllable's length.");
            println!("If this is not an error, it suggests that the meter is probably ramal.");
        } else if long_first_syllable_markers > 1 {
            long_first = true;
            println!("The first syllable in this meter now appears to be long.");
        } else if short_first_syllable_markers > 1 {
            short_first = true;
            println!("The first syllable in this meter now appears to be short.");
        } else {
            println!("Still insufficient evidence (<2) of a long vs. short first syllable…");
            println!("(It is easier to detect short syllables. Scant results may suggest long.)");
        }

        // Reassess second syllable length
        if long_second_syllable_markers > 0 && short_second_syllable_markers > 0 {
            println!("There are now contradictory indications of the second syllable's length.");
        } else if long_second_syllable_markers > 1 {
            long_second = true;
            println!("The second syllable in this meter now appears to be long.");
        } else if short_second_syllable_markers > 1 {
            short_second = true;
            println!("The second syllable in this meter now appears to be short.");
        } else {
            println!("Still insufficient evidence (<2) of a long vs. short second syllable…");
        }
    }

    // Report overall assessment
    assessment(
        long_meter,
        short_meter,
        long_first,
        short_first,
        long_second,
        short_second,
    );
}

fn reconstruct_hemistich(hemistich: String) -> Vec<char> {
    // Create a vector for reconstruction
    let mut hem_recon = Vec::new();

    // Go through one character at a time, passing valid input to the reconstruction
    for c in hemistich.trim().chars() {
        match c {
            // ٰVowels
            'ا' | 'آ' | 'و' | 'ی' => hem_recon.push(c),
            // Consonants (including isolated hamzah)
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => hem_recon.push(c),
            // Alif hamzah
            'أ' => hem_recon.push('ا'),
            // Vav hamzah
            'ؤ' => hem_recon.push('و'),
            // Yāʾ hamzah
            'ئ' => hem_recon.push('ی'),
            // Ignore hamzah diacritic
            'ٔ' => {}
            // Spaces can stay (in this version)
            ' ' => hem_recon.push(c),
            // ZWNJ becomes space
            '‌' => hem_recon.push(' '),

            // Flag anything else
            _ => println!("Unexpected character: {}", c.escape_unicode()),
        }
    }

    // Return the reconstructed hemistich
    hem_recon
}

fn long_first_syllable(hem_recon: Vec<char>) -> u32 {
    // Create integer variable to count markers
    let mut long_first_syllable_markers: u32 = 0;

    // Check for initial alif maddah, or alif as second character (including spaces)
    if hem_recon[0] == 'آ' || hem_recon[1] == 'آ' || hem_recon[1] == 'ا' {
        long_first_syllable_markers += 1;
    }

    // Check for initial "īn"
    if hem_recon[0..3] == ['ا', 'ی', 'ن'] {
        long_first_syllable_markers += 1;
    }

    // Check for initial "az," "bar," or "har" followed by a consonant
    if hem_recon[0..3] == ['ا', 'ز', ' ']
        || hem_recon[0..3] == ['ب', 'ر', ' ']
        || hem_recon[0..3] == ['ه', 'ر', ' ']
    {
        match hem_recon[3] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => long_first_syllable_markers += 1,
            _ => {}
        }
    }

    // Return number of markers
    long_first_syllable_markers
}

fn short_first_syllable(hem_recon: Vec<char>) -> u32 {
    // Create integer variable to count markers
    let mut short_first_syllable_markers: u32 = 0;

    // Check for initial "zih" followed by a consonant (after a space)
    if hem_recon[0..2] == ['ز', ' '] {
        match hem_recon[2] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => short_first_syllable_markers += 1,
            _ => {}
        }
    }

    // Check first three characters (including spaces)
    match hem_recon[0..3] {
        ['ب', 'ه', ' ']
        | ['ک', 'ه', ' ']
        | ['چ', 'و', ' ']
        | ['چ', 'ه', ' ']
        | ['ن', 'ه', ' ']
        | ['ک', 'ج', 'ا']
        | ['ه', 'م', 'ی']
        | ['خ', 'د', 'ا'] => short_first_syllable_markers += 1,
        _ => {}
    }

    // Check first four characters (including spaces)
    match hem_recon[0..4] {
        ['ا', 'گ', 'ر', ' ']
        | ['ش', 'و', 'د', ' ']
        | ['م', 'گ', 'ر', ' ']
        | ['چ', 'ر', 'ا', ' ']
        | ['م', 'ر', 'ا', ' ']
        | ['ت', 'ر', 'ا', ' ']
        | ['د', 'گ', 'ر', ' ']
        | ['ه', 'م', 'ه', ' ']
        | ['چ', 'ن', 'ا', 'ن']
        | ['چ', 'ن', 'ی', 'ن'] => short_first_syllable_markers += 1,
        _ => {}
    }

    // Return number of markers
    short_first_syllable_markers
}

fn long_second_syllable(hem_recon: Vec<char>, hem_recon_nospace: Vec<char>) -> u32 {
    // Create integer variable to count markers
    let mut long_second_syllable_markers: u32 = 0;

    // Check for alif maddah as the third character *or* letter
    if hem_recon[2] == 'آ' || hem_recon_nospace[2] == 'آ' {
        long_second_syllable_markers += 1;
    }

    // Check for alif as the third character, following a consonant
    if hem_recon[2] == 'ا' {
        match hem_recon[1] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => long_second_syllable_markers += 1,
            _ => {}
        }
    }

    // Return number of markers
    long_second_syllable_markers
}

fn short_second_syllable(hem_recon: Vec<char>, hem_recon_nospace: Vec<char>) -> u32 {
    // Create integer variable to count markers
    let mut short_second_syllable_markers: u32 = 0;

    // Set up vector windows including spaces
    let hem_windows_three = hem_recon.windows(3);

    // Set up vector windows excluding spaces
    let hem_letter_windows_three = hem_recon_nospace.windows(3);
    let hem_letter_windows_four = hem_recon_nospace.windows(4);

    // Test with windows of two characters (including spaces)
    for (counter, x) in hem_windows_three.enumerate() {
        if counter == 2 || counter == 3 {
            match x {
                ['ک', 'ه', ' '] | ['چ', 'ه', ' '] | ['ب', 'ه', ' '] | ['چ', 'و', ' '] => {
                    short_second_syllable_markers += 1
                }
                _ => {}
            }
        }
    }

    // Test with windows of three characters (excluding spaces)
    for (counter, x) in hem_letter_windows_three.enumerate() {
        if counter == 2 && (x == ['ک', 'ن', 'د'] || x == ['ش', 'و', 'د']) {
            short_second_syllable_markers += 1;
        }
    }

    // Test with windows of four characters (excluding spaces)
    for (counter, x) in hem_letter_windows_four.enumerate() {
        if counter == 2 && (x == ['چ', 'ن', 'ی', 'ن'] || x == ['چ', 'ن', 'ا', 'ن']) {
            short_second_syllable_markers += 1;
        }
    }

    // Return number of markers
    short_second_syllable_markers
}

fn initial_clues(hem_recon: Vec<char>) -> Vec<char> {
    // Create vector to hold clue identifiers
    let mut clue_identifiers = Vec::new();

    // Check for initial "agar" followed by consonant (ID 'a')
    if hem_recon[0..4] == ['ا', 'گ', 'ر', ' '] {
        match hem_recon[4] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => clue_identifiers.push('a'),
            _ => {}
        }
    }

    // Check for initial "kasī" followed by consonant (ID 'k')
    if hem_recon[0..4] == ['ک', 'س', 'ی', ' '] {
        match hem_recon[4] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => clue_identifiers.push('k'),
            _ => {}
        }
    }

    // Check for initial "yakī" followed by consonant (ID 'y')
    if hem_recon[0..4] == ['ی', 'ک', 'ی', ' '] {
        match hem_recon[4] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر'
            | 'ز' | 'ژ' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق'
            | 'ک' | 'گ' | 'ل' | 'م' | 'ن' | 'ه' => clue_identifiers.push('y'),
            _ => {}
        }
    }

    // Check for initial "chunīn" or "chunān" (ID 'c')
    if hem_recon[0..4] == ['چ', 'ن', 'ی', 'ن'] || hem_recon[0..4] == ['چ', 'ن', 'ا', 'ن'] {
        clue_identifiers.push('c');
    }

    // Return clue markers
    clue_identifiers
}

fn first_syllable_assessment(
    long_first_syllable_markers: u32,
    long_first_syllable_locs: String,
    short_first_syllable_markers: u32,
    short_first_syllable_locs: String,
) -> (bool, bool) {
    // Set up Booleans
    let mut long_first = false;
    let mut short_first = false;

    // Report indications of first syllable length
    println!("*** First syllable length ***");
    if long_first_syllable_markers > 0 {
        println!(
            "Indications of a long first syllable: {} (at {})",
            long_first_syllable_markers,
            long_first_syllable_locs.trim_end_matches(", ")
        );
    }
    if short_first_syllable_markers > 0 {
        println!(
            "Indications of a short first syllable: {} (at {})",
            short_first_syllable_markers,
            short_first_syllable_locs.trim_end_matches(", ")
        );
    }

    // Report assessment of first syllable length
    if long_first_syllable_markers > 0 && short_first_syllable_markers > 0 {
        println!("There are contradictory indications of a long vs. short first syllable.");
        println!("If this is not an error, it suggests that the meter is probably ramal.");
    } else if long_first_syllable_markers > 1 {
        long_first = true;
        println!("The first syllable in this meter appears to be long.");
    } else if short_first_syllable_markers > 1 {
        short_first = true;
        println!("The first syllable in this meter appears to be short.");
    } else {
        println!("So far, insufficient evidence (<2) of a long vs. short first syllable…");
        println!("(It is easier to detect short syllables. Scant results may suggest long.)");
    }

    // Return Boolean values
    (long_first, short_first)
}

fn second_syllable_assessment(
    long_second_syllable_markers: u32,
    long_second_syllable_locs: String,
    short_second_syllable_markers: u32,
    short_second_syllable_locs: String,
) -> (bool, bool) {
    // Set up Booleans
    let mut long_second = false;
    let mut short_second = false;

    // Report indications of second syllable length
    println!("*** Second syllable length ***");
    if long_second_syllable_markers > 0 {
        println!(
            "Suggestions of a long second syllable: {} (at {})",
            long_second_syllable_markers,
            long_second_syllable_locs.trim_end_matches(", ")
        );
        if long_second_syllable_markers == 1 {
            println!("(Be careful with this; it's a bit error-prone.)");
        }
    }
    if short_second_syllable_markers > 0 {
        println!(
            "Suggestions of a short second syllable: {} (at {})",
            short_second_syllable_markers,
            short_second_syllable_locs.trim_end_matches(", ")
        );
        if short_second_syllable_markers == 1 {
            println!("(Be careful with this; it's a bit error-prone.)");
        }
    }

    // Report assessment of second syllable length
    if long_second_syllable_markers > 0 && short_second_syllable_markers > 0 {
        println!("There are contradictory indications of a long vs. short second syllable.");
    } else if long_second_syllable_markers > 1 {
        long_second = true;
        println!("The second syllable in this meter appears to be long.");
    } else if short_second_syllable_markers > 1 {
        short_second = true;
        println!("The second syllable in this meter appears to be short.");
    } else {
        println!("So far, insufficient evidence (<2) of a long vs. short second syllable…");
    }

    // Return Boolean values
    (long_second, short_second)
}

fn assessment(
    long_meter: bool,
    short_meter: bool,
    long_first: bool,
    short_first: bool,
    long_second: bool,
    short_second: bool,
) {
    println!("*** Overall assessment ***");
    if long_meter {
        if long_first {
            if short_second {
                println!("Long meter, long first syllable, short second syllable?");
                println!("Consider ramal…");
            } else {
                println!("Long meter, long first syllable, indeterminate second syllable?");
                println!("Consider, with a long second syllable, hazaj or mużāriʿ (akhrab)…");
                println!("Consider, with a short second syllable, ramal…");
            }
        } else if short_first {
            if long_second {
                println!("Long meter, short first syllable, long second syllable?");
                println!("Consider, with a long third syllable, hazaj (sālim)…");
                println!("Consider, with a short third syllable, mujtaṡṡ…");
            } else if short_second {
                println!("Long meter, short first syllable, short second syllable?");
                println!("Consider ramal…");
            } else {
                println!("Long meter, short first syllable, indeterminate second syllable?");
                println!("Consider, with a long second syllable, hazaj (sālim) or mujtaṡṡ…");
                println!("Consider, with a short second syllable, ramal…");
            }
        } else {
            println!("What is clearest is that the meter appears to be long.");
            println!("If there were mixed signals about the first syllable, consider ramal…");
        }
    } else if short_meter {
        if long_first {
            if short_second {
                println!("Short meter, long first syllable, short second syllable?");
                println!("Consider khafīf or ramal…");
                println!("(These can be tricky to distinguish; khafīf is more common.)");
            } else {
                println!("Short meter, long first syllable, indeterminate second syllable?");
                println!("Consider, with a long second syllable, hazaj (akhrab)…");
                println!("Consider, with a short second syllable, khafīf or ramal…");
                println!("(The prior two can be tricky to distinguish; khafīf is more common.)");
            }
        } else if short_first {
            if short_second {
                println!("Short meter, short first syllable, short second syllable?");
                println!("This would be rare. Consider ramal…");
            } else {
                println!("Short meter, short first syllable, indeterminate second syllable?");
                println!("Consider hazaj (musaddas) or mutaqārib (muṡamman)…");
            }
        } else {
            println!("What is clearest is that the meter appears to be short.");
            println!("If there were mixed signals about the first syllable, consider ramal…");
        }
    } else {
        println!("With the meter length unclear, no further conclusions will be drawn.");
    }
}
