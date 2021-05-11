use anyhow::{anyhow, Result};
use regex::Regex;
use std::{env, fs};

fn main() -> Result<()> {
    // Get filename from command arguments
    let args: Vec<String> = env::args().collect();

    // Return error if no argument was given
    if args.len() < 2 {
        return Err(anyhow!("A filename must be provided"));
    }

    // Define variable for file path
    let filename = &args[1];

    // Apply a sanity check for the size of the file provided
    let file_size = fs::metadata(filename)?.len();
    if file_size > 10_000 {
        return Err(anyhow!("The file appears suspiciously large"));
    }

    // Read the relevant file to a string
    let poem = fs::read_to_string(filename)?;

    // Trim outside whitespace and remove interior empty lines
    let re = Regex::new("\n{2,}").unwrap();
    let poem_trimmed = re.replace_all(poem.trim(), "\n");

    // Return error if poem is too short
    if poem_trimmed.lines().count() < 10 {
        return Err(anyhow!("At least ten hemistichs are required"));
    }

    // Initialize Booleans for meter length classification
    let mut long_meter = false;
    let mut short_meter = false;

    // Initialize Booleans for other clues
    let mut found_initial_clues = false;
    let mut initial_agar = false;
    let mut initial_chun = false;
    let mut initial_kasi = false;
    let mut initial_yaki = false;

    // Initialize integer variable to count letters
    let mut total_letters: u32 = 0;

    // Initialize variables for checking syllable length
    let mut long_first_syl_markers: u32 = 0;
    let mut long_first_syl_locs = String::new();
    let mut short_first_syl_markers: u32 = 0;
    let mut short_first_syl_locs = String::new();
    let mut new_short_first_syl_markers: u32 = 0;
    let mut long_second_syl_markers: u32 = 0;
    let mut long_second_syl_locs = String::new();
    let mut new_long_second_syl_markers: u32 = 0;
    let mut short_second_syl_markers: u32 = 0;
    let mut short_second_syl_locs = String::new();

    // Setup for printing reconstructed hemistichs
    println!("*** Assessing the following hemistichs ***");

    //
    // Primary loop
    //

    for (i, hem) in poem_trimmed.lines().enumerate() {
        // Take only the first ten hemistichs
        if i > 9 {
            continue;
        }

        // Define a non-zero-indexed counter for display
        let hem_no = i + 1;

        // Reconstruct hemistich as a vector of chars; create a version without spaces
        let hem_reconst: Vec<char> = reconstruct_hemistich(hem.to_string())?;
        let mut hem_nospace = hem_reconst.clone();
        hem_nospace.retain(|x| *x != ' ');

        // Print reconstructed hemistich and its number
        let hem_reconst_str: String = hem_reconst.iter().collect();
        println!("{}: {}", hem_no, hem_reconst_str);

        // Count chars (excluding spaces); add to the total
        let hem_letter_count = hem_nospace.len();
        total_letters += hem_letter_count as u32;

        // Check for long first syllable
        let long_first_syl_results = long_first_syllable(&hem_reconst);
        if long_first_syl_results > 0 {
            long_first_syl_markers += long_first_syl_results;
            long_first_syl_locs.push_str(&hem_no.to_string());
            long_first_syl_locs.push_str(", ");
        }

        // Check for short first syllable
        let short_first_syllable_results = short_first_syllable(&hem_reconst);
        if short_first_syllable_results > 0 {
            short_first_syl_markers += short_first_syllable_results;
            short_first_syl_locs.push_str(&hem_no.to_string());
            short_first_syl_locs.push_str(", ");
        }

        // Check for long second syllable
        let long_second_syllable_results = long_second_syllable(&hem_reconst, &hem_nospace);
        if long_second_syllable_results > 0 {
            long_second_syl_markers += long_second_syllable_results;
            long_second_syl_locs.push_str(&hem_no.to_string());
            long_second_syl_locs.push_str(", ");
        }

        // Check for short second syllable
        let short_second_syllable_results = short_second_syllable(&hem_reconst, &hem_nospace);
        if short_second_syllable_results > 0 {
            short_second_syl_markers += short_second_syllable_results;
            short_second_syl_locs.push_str(&hem_no.to_string());
            short_second_syl_locs.push_str(", ");
        }

        // Check for hemistich-initial clues
        // 'a' = initial "agar" followed by a consonant (after a space)
        // 'c' = initial "chunīn" or "chunān"
        // 'k' = initial "kasī" followed by a consonant (after a space)
        // 'y' = initial "yakī" followed by a consonant (after a space)
        let initial_clues_results = initial_clues(&hem_reconst);
        if !initial_clues_results.is_empty() {
            found_initial_clues = true;
            for c in initial_clues_results.iter() {
                match *c {
                    'a' => {
                        initial_agar = true;
                        new_long_second_syl_markers += 1;
                    }
                    'c' => {
                        initial_chun = true;
                        new_long_second_syl_markers += 1;
                    }
                    'k' => {
                        initial_kasi = true;
                        new_short_first_syl_markers += 1;
                        new_long_second_syl_markers += 1;
                    }
                    'y' => {
                        initial_yaki = true;
                        new_short_first_syl_markers += 1;
                        new_long_second_syl_markers += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    //
    // Results
    //

    // Calculate average letters per hemistich
    let total_letters_float = f64::from(total_letters);
    let avg_letters = total_letters_float / 10.0;

    // Report assessment of meter length
    println!("*** Meter length ***");
    println!("Average letters per hemistich: {:.1}", avg_letters);
    if avg_letters >= 23.0 {
        long_meter = true;
        println!("The meter appears to be long (muṡamman).");
    } else if avg_letters >= 21.0 {
        println!("It's not obvious whether the meter is long or short.");
        println!("(In this gray area, the answer is usually long.)");
    } else {
        short_meter = true;
        println!("The meter appears to be short (musaddas; or mutaqārib muṡamman).");
    }

    // Report assessment of first syllable length
    let (mut long_first, mut short_first) = first_syllable_assessment(
        long_first_syl_markers,
        long_first_syl_locs,
        short_first_syl_markers,
        short_first_syl_locs,
    );

    // Report assessment of second syllable length
    let (mut long_second, mut short_second) = second_syllable_assessment(
        long_second_syl_markers,
        long_second_syl_locs,
        short_second_syl_markers,
        short_second_syl_locs,
    );

    // Address other hemistich-initial clues, if any
    if found_initial_clues {
        initial_clues_assessment(initial_agar, initial_chun, initial_kasi, initial_yaki);

        // Add new syllable length markers
        short_first_syl_markers += new_short_first_syl_markers;
        long_second_syl_markers += new_long_second_syl_markers;
    }

    // Reassess first syllable length, if applicable
    if new_short_first_syl_markers > 0 {
        if long_first_syl_markers > 0 && short_first_syl_markers > 0 {
            println!("There are now contradictory indications of the first syllable's length.");
            println!("If this is not an error, it suggests that the meter is probably ramal.");
        } else if long_first_syl_markers > 1 {
            long_first = true;
            println!("The first syllable in this meter now appears to be long.");
        } else if short_first_syl_markers > 1 {
            short_first = true;
            println!("The first syllable in this meter now appears to be short.");
        } else {
            println!("Still insufficient evidence (<2) of a long vs. short first syllable…");
            println!("(It's easier to detect short syllables. Scant results may suggest long.)");
        }
    }

    // Reassess second syllable length, if applicable
    if new_long_second_syl_markers > 0 {
        if long_second_syl_markers > 0 && short_second_syl_markers > 0 {
            println!("There are now contradictory indications of the second syllable's length.");
        } else if long_second_syl_markers > 1 {
            long_second = true;
            println!("The second syllable in this meter now appears to be long.");
        } else if short_second_syl_markers > 1 {
            short_second = true;
            println!("The second syllable in this meter now appears to be short.");
        } else {
            println!("Still insufficient evidence (<2) of a long vs. short second syllable…");
        }
    }

    // Report overall assessment
    final_assessment(
        long_meter,
        short_meter,
        long_first,
        short_first,
        long_second,
        short_second,
    );

    Ok(())
}

//
// Analysis functions
//

fn reconstruct_hemistich(hem: String) -> Result<Vec<char>> {
    // Create a vec for reconstruction
    let mut hem_reconst = Vec::new();

    // Go through one character at a time, passing valid input to the reconstruction
    for c in hem.trim().chars() {
        match c {
            // ٰVowels
            'ا' | 'آ' | 'و' | 'ی' => hem_reconst.push(c),
            // Consonants (including isolated hamzah)
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => hem_reconst.push(c),
            // Alif hamzah
            'أ' => hem_reconst.push('ا'),
            // Vav hamzah
            'ؤ' => hem_reconst.push('و'),
            // Yāʾ hamzah
            'ئ' => hem_reconst.push('ی'),
            // Ignore hamzah diacritic
            'ٔ' => {}
            // Spaces can stay (in this version)
            ' ' => hem_reconst.push(c),
            // ZWNJ becomes space
            '‌' => hem_reconst.push(' '),

            // Flag anything else
            _ => {
                println!("An unexpected character was found: {}", c.escape_unicode());
                println!("Please notify the developer if you think this is a bug.");
                return Err(anyhow!("Text must be in Persian/Arabic script"));
            }
        }
    }

    Ok(hem_reconst)
}

fn long_first_syllable(hem_reconst: &[char]) -> u32 {
    // Mutable u32 to count markers
    let mut long_first_syl_markers: u32 = 0;

    // Check for initial alif maddah, or alif as second character (incl. spaces)
    if hem_reconst[0] == 'آ' || hem_reconst[1] == 'آ' || hem_reconst[1] == 'ا' {
        long_first_syl_markers += 1;
    }

    // Check for initial "īn"
    if hem_reconst[0..3] == ['ا', 'ی', 'ن'] {
        long_first_syl_markers += 1;
    }

    // Check for initial "az," "bar," or "har" followed by consonant
    if hem_reconst[0..3] == ['ا', 'ز', ' ']
        || hem_reconst[0..3] == ['ب', 'ر', ' ']
        || hem_reconst[0..3] == ['ه', 'ر', ' ']
    {
        match hem_reconst[3] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => long_first_syl_markers += 1,
            _ => {}
        }
    }

    long_first_syl_markers
}

fn short_first_syllable(hem_reconst: &[char]) -> u32 {
    // Mutable u32 to count markers
    let mut short_first_syl_markers: u32 = 0;

    // Check for initial "zih" followed by consonant (after a space)
    if hem_reconst[0..2] == ['ز', ' '] {
        match hem_reconst[2] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => short_first_syl_markers += 1,
            _ => {}
        }
    }

    // Check first three characters (incl. spaces)
    match hem_reconst[0..3] {
        ['ب', 'ه', ' ']
        | ['ک', 'ه', ' ']
        | ['چ', 'و', ' ']
        | ['چ', 'ه', ' ']
        | ['ن', 'ه', ' ']
        | ['ک', 'ج', 'ا']
        | ['ه', 'م', 'ی']
        | ['خ', 'د', 'ا'] => short_first_syl_markers += 1,
        _ => {}
    }

    // Check first four characters (incl. spaces)
    match hem_reconst[0..4] {
        ['ا', 'گ', 'ر', ' ']
        | ['ش', 'و', 'د', ' ']
        | ['م', 'گ', 'ر', ' ']
        | ['چ', 'ر', 'ا', ' ']
        | ['م', 'ر', 'ا', ' ']
        | ['ت', 'ر', 'ا', ' ']
        | ['د', 'گ', 'ر', ' ']
        | ['ه', 'م', 'ه', ' ']
        | ['چ', 'ن', 'ا', 'ن']
        | ['چ', 'ن', 'ی', 'ن'] => short_first_syl_markers += 1,
        _ => {}
    }

    short_first_syl_markers
}

fn long_second_syllable(hem_reconst: &[char], hem_nospace: &[char]) -> u32 {
    // Mutable u32 to count markers
    let mut long_second_syl_markers: u32 = 0;

    // Check for alif maddah as third character *or* letter
    if hem_reconst[2] == 'آ' || hem_nospace[2] == 'آ' {
        long_second_syl_markers += 1;
    }

    // Check for alif as third character, following consonant
    if hem_reconst[2] == 'ا' {
        match hem_reconst[1] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => long_second_syl_markers += 1,
            _ => {}
        }
    }

    long_second_syl_markers
}

fn short_second_syllable(hem_reconst: &[char], hem_nospace: &[char]) -> u32 {
    // Mutable u32 to count markers
    let mut short_second_syl_markers: u32 = 0;

    // Set up vector windows incl. spaces
    let hem_windows_three = hem_reconst.windows(3);

    // Set up vector windows excl. spaces
    let letter_windows_three = hem_nospace.windows(3);
    let letter_windows_four = hem_nospace.windows(4);

    // Test with windows of two letters (plus a space)
    for (i, x) in hem_windows_three.enumerate() {
        if i == 2 || i == 3 {
            match x {
                ['ک', 'ه', ' '] | ['چ', 'ه', ' '] | ['ب', 'ه', ' '] | ['چ', 'و', ' '] => {
                    short_second_syl_markers += 1
                }
                _ => {}
            }
        }
    }

    // Test with windows of three letters (excl. spaces)
    for (i, x) in letter_windows_three.enumerate() {
        if i == 2 && (x == ['ک', 'ن', 'د'] || x == ['ش', 'و', 'د']) {
            short_second_syl_markers += 1;
        }
    }

    // Test with windows of four letters (excl. spaces)
    for (i, x) in letter_windows_four.enumerate() {
        if i == 2 && (x == ['چ', 'ن', 'ی', 'ن'] || x == ['چ', 'ن', 'ا', 'ن']) {
            short_second_syl_markers += 1;
        }
    }

    short_second_syl_markers
}

fn initial_clues(hem_reconst: &[char]) -> Vec<char> {
    // Create vector to hold clue identifiers
    let mut clue_ids = Vec::new();

    // Check for initial "agar" followed by consonant (ID 'a')
    if hem_reconst[0..4] == ['ا', 'گ', 'ر', ' '] {
        match hem_reconst[4] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => clue_ids.push('a'),
            _ => {}
        }
    }

    // Check for initial "kasī" followed by consonant (ID 'k')
    if hem_reconst[0..4] == ['ک', 'س', 'ی', ' '] {
        match hem_reconst[4] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => clue_ids.push('k'),
            _ => {}
        }
    }

    // Check for initial "yakī" followed by consonant (ID 'y')
    if hem_reconst[0..4] == ['ی', 'ک', 'ی', ' '] {
        match hem_reconst[4] {
            // Consonants
            'ء' | 'ب' | 'پ' | 'ت' | 'ث' | 'ج' | 'چ' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ'
            | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' | 'ق' | 'ک' | 'گ' | 'ل' | 'م'
            | 'ن' | 'ه' => clue_ids.push('y'),
            _ => {}
        }
    }

    // Check for initial "chunīn" or "chunān" (ID 'c')
    if hem_reconst[0..4] == ['چ', 'ن', 'ی', 'ن'] || hem_reconst[0..4] == ['چ', 'ن', 'ا', 'ن']
    {
        clue_ids.push('c');
    }

    clue_ids
}

//
// Results functions
//

fn first_syllable_assessment(
    long_first_syl_markers: u32,
    long_first_syl_locs: String,
    short_first_syl_markers: u32,
    short_first_syl_locs: String,
) -> (bool, bool) {
    // Initialize bools
    let mut long_first = false;
    let mut short_first = false;

    // Report indications of first syllable length
    println!("*** First syllable length ***");
    if long_first_syl_markers > 0 {
        println!(
            "Indications of a long first syllable: {} (at {})",
            long_first_syl_markers,
            long_first_syl_locs.trim_end_matches(", ")
        );
    }
    if short_first_syl_markers > 0 {
        println!(
            "Indications of a short first syllable: {} (at {})",
            short_first_syl_markers,
            short_first_syl_locs.trim_end_matches(", ")
        );
    }

    // Report assessment of first syllable length
    if long_first_syl_markers > 0 && short_first_syl_markers > 0 {
        println!("There are contradictory indications of a long vs. short first syllable.");
        println!("If this is not an error, it suggests that the meter is probably ramal.");
    } else if long_first_syl_markers > 1 {
        long_first = true;
        println!("The first syllable in this meter appears to be long.");
    } else if short_first_syl_markers > 1 {
        short_first = true;
        println!("The first syllable in this meter appears to be short.");
    } else {
        println!("So far, insufficient evidence (<2) of a long vs. short first syllable…");
        println!("(It's easier to detect short syllables. Scant results may suggest long.)");
    }

    (long_first, short_first)
}

fn second_syllable_assessment(
    long_second_syl_markers: u32,
    long_second_syl_locs: String,
    short_second_syl_markers: u32,
    short_second_syl_locs: String,
) -> (bool, bool) {
    // Initialize bools
    let mut long_second = false;
    let mut short_second = false;

    // Report indications of second syllable length
    println!("*** Second syllable length ***");
    if long_second_syl_markers > 0 {
        println!(
            "Suggestions of a long second syllable: {} (at {})",
            long_second_syl_markers,
            long_second_syl_locs.trim_end_matches(", ")
        );
        if long_second_syl_markers == 1 {
            println!("(Be careful with this; it's a bit error-prone.)");
        }
    }
    if short_second_syl_markers > 0 {
        println!(
            "Suggestions of a short second syllable: {} (at {})",
            short_second_syl_markers,
            short_second_syl_locs.trim_end_matches(", ")
        );
        if short_second_syl_markers == 1 {
            println!("(Be careful with this; it's a bit error-prone.)");
        }
    }

    // Report assessment of second syllable length
    if long_second_syl_markers > 0 && short_second_syl_markers > 0 {
        println!("There are contradictory indications of a long vs. short second syllable.");
    } else if long_second_syl_markers > 1 {
        long_second = true;
        println!("The second syllable in this meter appears to be long.");
    } else if short_second_syl_markers > 1 {
        short_second = true;
        println!("The second syllable in this meter appears to be short.");
    } else {
        println!("So far, insufficient evidence (<2) of a long vs. short second syllable…");
    }

    (long_second, short_second)
}

fn initial_clues_assessment(
    initial_agar: bool,
    initial_chun: bool,
    initial_kasi: bool,
    initial_yaki: bool,
) {
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
}

fn final_assessment(
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
                println!("Consider ramal.");
            } else {
                println!("Long meter, long first syllable, indeterminate second syllable?");
                println!("Consider, with a long second syllable, hazaj or mużāriʿ (akhrab).");
                println!("Consider, with a short second syllable, ramal.");
            }
        } else if short_first {
            if long_second {
                println!("Long meter, short first syllable, long second syllable?");
                println!("Consider, with a long third syllable, hazaj (sālim).");
                println!("Consider, with a short third syllable, mujtaṡṡ.");
            } else if short_second {
                println!("Long meter, short first syllable, short second syllable?");
                println!("Consider ramal.");
            } else {
                println!("Long meter, short first syllable, indeterminate second syllable?");
                println!("Consider, with a long second syllable, hazaj (sālim) or mujtaṡṡ.");
                println!("Consider, with a short second syllable, ramal.");
            }
        } else {
            println!("What is clearest is that the meter appears to be long.");
            println!("If there were mixed signals about the first syllable, consider ramal.");
        }
    } else if short_meter {
        if long_first {
            if short_second {
                println!("Short meter, long first syllable, short second syllable?");
                println!("Consider khafīf or ramal.");
                println!("(These can be tricky to distinguish. Khafīf is more common.)");
            } else {
                println!("Short meter, long first syllable, indeterminate second syllable?");
                println!("Consider, with a long second syllable, hazaj (akhrab).");
                println!("Consider, with a short second syllable, khafīf or ramal.");
                println!("(The prior two can be tricky to distinguish. Khafīf is more common.)");
            }
        } else if short_first {
            if short_second {
                println!("Short meter, short first syllable, short second syllable?");
                println!("This would be rare. Consider ramal.");
            } else {
                println!("Short meter, short first syllable, indeterminate second syllable?");
                println!("Consider hazaj (musaddas) or mutaqārib (muṡamman).");
            }
        } else {
            println!("What is clearest is that the meter appears to be short.");
            println!("If there were mixed signals about the first syllable, consider ramal.");
        }
    } else {
        println!("With the meter length unclear, no further conclusions will be drawn.");
    }
}
