use anyhow::{anyhow, Result};
use clap::Parser;
use regex::Regex;
use std::fs;

/// A program that attempts to find the meter of a Persian poem
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of input text file
    #[clap()]
    input: String,
}

fn main() -> Result<()> {
    //
    // Argument parsing etc.
    //

    // Parse args; get input file path
    let args = Args::parse();
    let path = &args.input;

    // Apply a sanity check for the size of the file provided
    let file_size = fs::metadata(path)?.len();
    if file_size > 10_000 {
        return Err(anyhow!("The file appears suspiciously large"));
    }

    // Read file to string
    let poem = fs::read_to_string(path)?;

    // Trim outside whitespace and remove interior empty lines
    let re = Regex::new("\n{2,}").unwrap();
    let poem_trimmed = re.replace_all(poem.trim(), "\n");

    // Error out if poem is too short
    let total_hemistichs = poem_trimmed.lines().count();
    if total_hemistichs < 10 {
        return Err(anyhow!("At least ten hemistichs are required"));
    }

    //
    // Global variables
    //

    // Booleans for meter length classification
    let mut long_meter = false;
    let mut short_meter = false;

    // Variable to count letters
    let mut total_letters: u32 = 0;

    // Variables for checking individual syllable lengths
    let mut long_first_syl_markers: u32 = 0;
    let mut long_first_syl_locs = String::new();
    let mut short_first_syl_markers: u32 = 0;
    let mut short_first_syl_locs = String::new();
    let mut long_second_syl_markers: u32 = 0;
    let mut long_second_syl_locs = String::new();
    let mut short_second_syl_markers: u32 = 0;
    let mut short_second_syl_locs = String::new();

    // Variable for results report to be printed or saved
    let mut results_report = String::from("*** Assessing the following hemistichs ***\n");

    //
    // Primary loop
    //

    for (i, hem) in poem_trimmed.lines().enumerate() {
        // Take at most forty hemistichs (i.e., twenty lines)
        if i > 39 {
            continue;
        }

        // Non-zero-indexed counter for display
        let hem_no = i + 1;

        // Reconstruct hemistich as vector of chars
        // Make a second version without spaces
        let hem_reconst: Vec<char> = reconstruct_hemistich(hem.to_string())?;
        let mut hem_nospace = hem_reconst.clone();
        hem_nospace.retain(|x| *x != ' ');

        // Record reconstructed hemistich and its number
        let hem_reconst_str: String = hem_reconst.iter().collect();
        results_report += &format!("{}: {}\n", hem_no, hem_reconst_str);

        // Count chars (excluding spaces); add to the total
        let hem_letter_count = hem_nospace.len() as u32;
        total_letters += hem_letter_count;

        // Check for long first syllable
        if long_first_syllable(&hem_reconst) {
            long_first_syl_markers += 1;
            long_first_syl_locs.push_str(&hem_no.to_string());
            long_first_syl_locs.push_str(", ");
        }

        // Check for short first syllable
        if short_first_syllable(&hem_reconst) {
            short_first_syl_markers += 1;
            short_first_syl_locs.push_str(&hem_no.to_string());
            short_first_syl_locs.push_str(", ");
        }

        // Check for long second syllable
        if long_second_syllable(&hem_reconst) {
            long_second_syl_markers += 1;
            long_second_syl_locs.push_str(&hem_no.to_string());
            long_second_syl_locs.push_str(", ");
        }

        // Check for short second syllable
        if short_second_syllable(&hem_reconst, &hem_nospace) {
            short_second_syl_markers += 1;
            short_second_syl_locs.push_str(&hem_no.to_string());
            short_second_syl_locs.push_str(", ");
        }

        if let Some(result) = initial_clues(&hem_reconst) {
            match result {
                "chunin/an" | "kasi" | "yaki" => {
                    short_first_syl_markers += 1;
                    short_first_syl_locs.push_str(&hem_no.to_string());
                    short_first_syl_locs.push_str(", ");

                    long_second_syl_markers += 1;
                    long_second_syl_locs.push_str(&hem_no.to_string());
                    long_second_syl_locs.push_str(", ");
                }
                "chist" | "dust" | "nist" | "hamchu" => {
                    long_first_syl_markers += 1;
                    long_first_syl_locs.push_str(&hem_no.to_string());
                    long_first_syl_locs.push_str(", ");

                    short_second_syl_markers += 1;
                    short_second_syl_locs.push_str(&hem_no.to_string());
                    short_second_syl_locs.push_str(", ");
                }
                _ => {}
            }
        }
    }

    //
    // Results
    //

    // Calculate average letters per hemistich
    let total_letters_float = total_letters as f64;

    let total_hemistichs_float = if total_hemistichs > 40 {
        40.0
    } else {
        total_hemistichs as f64
    };

    let avg_letters = total_letters_float / total_hemistichs_float;

    // Report assessment of meter length
    results_report += "*** Meter length ***\n";
    results_report += &format!("Average letters per hemistich: {:.1}\n", avg_letters);

    if avg_letters >= 23.5 {
        long_meter = true;
        results_report += "The meter appears to be long (muṡamman).\n";
    } else if avg_letters >= 22.5 {
        long_meter = true;
        results_report += "The meter appears to be long (muṡamman).\n";
        results_report += "(But this is pretty short for a long meter!)\n";
    } else if avg_letters >= 21.0 {
        short_meter = true;
        results_report += "The meter appears to be short (musaddas; or mutaqārib muṡamman).\n";
        results_report += "(But this is pretty long for a short meter!)\n";
    } else {
        short_meter = true;
        results_report += "The meter appears to be short (musaddas; or mutaqārib muṡamman).\n";
    }

    // Report assessment of first syllable length
    let (long_first, short_first, first_report) = first_syllable_assessment(
        long_first_syl_markers,
        long_first_syl_locs,
        short_first_syl_markers,
        short_first_syl_locs,
    );

    results_report += &first_report;

    // Report assessment of second syllable length
    let (long_second, short_second, second_report) = second_syllable_assessment(
        long_second_syl_markers,
        long_second_syl_locs,
        short_second_syl_markers,
        short_second_syl_locs,
    );

    results_report += &second_report;

    // Report overall assessment
    let summary_report = final_assessment(
        long_meter,
        short_meter,
        long_first,
        short_first,
        long_second,
        short_second,
    );

    results_report += &summary_report;
    print!("{}", results_report);

    Ok(())
}

//
// Analysis functions
//

fn reconstruct_hemistich(hem: String) -> Result<Vec<char>> {
    // Create a vec for reconstruction
    let mut hem_reconst = Vec::new();

    // Review one character at a time, passing through valid input
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
            // Vāv hamzah
            'ؤ' => hem_reconst.push('و'),
            // Yā’ hamzah
            'ئ' => hem_reconst.push('ی'),
            // Replace tā’ marbūṭah with hā’
            'ة' => hem_reconst.push('ه'),
            // Ignore hamzah diacritic, fatḥah, shaddah, ḍammah, kasrah, sukūn,
            // tanwīn fatḥah, dagger alif
            'ٔ' | 'َ' | 'ّ' | 'ُ' | 'ِ' | 'ْ' | 'ً' | 'ٰ' => {}
            // Spaces can stay (for now)
            ' ' => hem_reconst.push(c),
            // ZWNJ becomes space
            '‌' => hem_reconst.push(' '),
            // Ignore comma, question mark, or exclamation mark
            '،' | '؟' | '!' => {}

            // Flag anything else
            _ => {
                eprintln!("An unexpected character was found: {}", c.escape_unicode());
                eprintln!("Please notify the developer if you think this is a bug.");
                return Err(anyhow!("Text must be fully in Persian/Arabic script"));
            }
        }
    }

    Ok(hem_reconst)
}

fn long_first_syllable(hem_reconst: &[char]) -> bool {
    // Check for initial alif maddah, or alif as second character
    if hem_reconst[0] == 'آ' || hem_reconst[1] == 'ا' {
        return true;
    }

    let initial_three = &hem_reconst[0..3];

    // Check for initial "īn"
    if initial_three == ['ا', 'ی', 'ن'] {
        return true;
    }

    // Check for initial "khwā-"
    // I found only one word that would break this: "khavāniq"
    // But that's vanishingly rare
    if initial_three == ['خ', 'و', 'ا'] {
        return true;
    }

    let consonants = [
        'ء', 'ب', 'پ', 'ت', 'ث', 'ج', 'چ', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'ژ', 'س', 'ش', 'ص', 'ض',
        'ط', 'ظ', 'ع', 'غ', 'ف', 'ق', 'ک', 'گ', 'ل', 'م', 'ن', 'ه',
    ];

    // Check for initial "az," "bar," "har," "gar," or "ay" followed by a
    // consonant
    if (initial_three == ['ا', 'ز', ' ']
        || initial_three == ['ب', 'ر', ' ']
        || initial_three == ['ه', 'ر', ' ']
        || initial_three == ['گ', 'ر', ' ']
        || initial_three == ['ا', 'ی', ' '])
        && consonants.contains(&hem_reconst[3])
    {
        return true;
    }

    false
}

fn short_first_syllable(hem_reconst: &[char]) -> bool {
    let consonants = [
        'ء', 'ب', 'پ', 'ت', 'ث', 'ج', 'چ', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'ژ', 'س', 'ش', 'ص', 'ض',
        'ط', 'ظ', 'ع', 'غ', 'ف', 'ق', 'ک', 'گ', 'ل', 'م', 'ن', 'ه',
    ];

    // Check for initial "zih" followed by a consonant (after a space)
    if hem_reconst[0..2] == ['ز', ' '] && consonants.contains(&hem_reconst[2]) {
        return true;
    }

    // Check first three characters
    // Initial "bih" (risky?), "kih," "chu," "chih," or "nah" (risky?) followed
    // by a space
    // Initial "kujā," "hamī," "khudā," "agar," "chirā," or "digar"
    match hem_reconst[0..3] {
        ['ب', 'ه', ' ']
        | ['ک', 'ه', ' ']
        | ['چ', 'و', ' ']
        | ['چ', 'ه', ' ']
        | ['ن', 'ه', ' ']
        | ['ک', 'ج', 'ا']
        | ['ه', 'م', 'ی']
        | ['خ', 'د', 'ا']
        | ['ا', 'گ', 'ر']
        | ['چ', 'ر', 'ا']
        | ['د', 'گ', 'ر'] => return true,
        _ => {}
    }

    // Check first four characters
    // Initial "shavad," "magar," "marā,"" "turā," or "hamah" followed by a
    // space
    match hem_reconst[0..4] {
        ['ش', 'و', 'د', ' ']
        | ['م', 'گ', 'ر', ' ']
        | ['م', 'ر', 'ا', ' ']
        | ['ت', 'ر', 'ا', ' ']
        | ['ه', 'م', 'ه', ' '] => return true,
        _ => {}
    }

    false
}

fn long_second_syllable(hem_reconst: &[char]) -> bool {
    let consonants = [
        'ء', 'ب', 'پ', 'ت', 'ث', 'ج', 'چ', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'ژ', 'س', 'ش', 'ص', 'ض',
        'ط', 'ظ', 'ع', 'غ', 'ف', 'ق', 'ک', 'گ', 'ل', 'م', 'ن', 'ه',
    ];

    // Check for alif as third character, non-word-initial, not after vāv
    // Should maybe work on better criteria for alif qua long vowel marker
    if hem_reconst[2] == 'ا' && hem_reconst[1] != ' ' && hem_reconst[1] != 'و' {
        return true;
    }

    // Check for initial "agar" followed by consonant
    // This would already have been flagged for a short first syllable
    if hem_reconst[0..4] == ['ا', 'گ', 'ر', ' '] && consonants.contains(&hem_reconst[4]) {
        return true;
    }

    // Check for initial "bāshad" or "sāqī" followed by consonant
    // These would already have been flagged for a long first syllable
    if (hem_reconst[0..5] == ['ب', 'ا', 'ش', 'د', ' ']
        || hem_reconst[0..5] == ['س', 'ا', 'ق', 'ی', ' '])
        && consonants.contains(&hem_reconst[5])
    {
        return true;
    }

    // If the opening word is anything like "tā," "bā," "yā," etc., check if
    // what follows is clearly another long syllable
    if hem_reconst[1..3] == ['ا', ' '] && long_first_syllable(&hem_reconst[3..]) {
        return true;
    }

    // If the opening word is "ay," "gar," or "az," followed by a consonant,
    // check if what follows is clearly another long syllable
    if (hem_reconst[0..3] == ['ا', 'ی', ' ']
        || hem_reconst[0..3] == ['گ', 'ر', ' ']
        || hem_reconst[0..3] == ['ا', 'ز', ' '])
        && consonants.contains(&hem_reconst[3])
        && long_first_syllable(&hem_reconst[3..])
    {
        return true;
    }

    // If the opening word is "bih" or "kih" (short), check if what follows is
    // clearly a long syllable
    // Is this legit? It's worth a shot
    if (hem_reconst[0..3] == ['ب', 'ه', ' '] || hem_reconst[0..3] == ['ک', 'ه', ' '])
        && long_first_syllable(&hem_reconst[3..])
    {
        return true;
    }

    false
}

fn short_second_syllable(hem_reconst: &[char], hem_nospace: &[char]) -> bool {
    // If the opening word is "bih" or "kih" (very common), check if what
    // follows is clearly another short syllable
    if (hem_reconst[0..3] == ['ب', 'ه', ' '] || hem_reconst[0..3] == ['ک', 'ه', ' '])
        && short_first_syllable(&hem_reconst[3..])
    {
        return true;
    }

    // If the opening word is anything like "tā," "bā," "yā," etc., check if
    // what follows is clearly a short syllable
    if hem_reconst[1..3] == ['ا', ' '] && short_first_syllable(&hem_reconst[3..]) {
        return true;
    }

    let initial_five = &hem_reconst[0..5];
    let initial_six = &hem_reconst[0..6];

    // Some of the below imply a long first syllable that would not have been
    // caught otherwise. Such cases should be dealt with instead in "initial
    // clues"

    // Check for initial "har-kih," "ān-kih," "gar-chih," or "ān-chih" (with or
    // without a space)
    if initial_five == ['ه', 'ر', 'ک', 'ه', ' ']
        || initial_six == ['ه', 'ر', ' ', 'ک', 'ه', ' ']
        || initial_five == ['آ', 'ن', 'ک', 'ه', ' ']
        || initial_six == ['آ', 'ن', ' ', 'ک', 'ه', ' ']
        || initial_five == ['گ', 'ر', 'چ', 'ه', ' ']
        || initial_six == ['گ', 'ر', ' ', 'چ', 'ه', ' ']
        || initial_five == ['آ', 'ن', 'چ', 'ه', ' ']
        || initial_six == ['آ', 'ن', ' ', 'چ', 'ه', ' ']
    {
        return true;
    }

    // Used to check here for near-initial "kunad" or "shavad"
    // Could try to bring that back somehow?

    // Check for "chunīn" or "chunān" starting at the third letter (with or
    // without a space). I think this is valid
    if hem_nospace[2..6] == ['چ', 'ن', 'ی', 'ن'] || hem_nospace[2..6] == ['چ', 'ن', 'ا', 'ن']
    {
        return true;
    }

    false
}

fn initial_clues(hem_reconst: &[char]) -> Option<&str> {
    let consonants = [
        'ء', 'ب', 'پ', 'ت', 'ث', 'ج', 'چ', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'ژ', 'س', 'ش', 'ص', 'ض',
        'ط', 'ظ', 'ع', 'غ', 'ف', 'ق', 'ک', 'گ', 'ل', 'م', 'ن', 'ه',
    ];

    let initial_four = &hem_reconst[0..4];
    let initial_five = &hem_reconst[0..5];
    let initial_six = &hem_reconst[0..6];

    // Check for initial "kasī" followed by consonant
    if initial_four == ['ک', 'س', 'ی', ' '] && consonants.contains(&hem_reconst[4]) {
        return Some("kasi");
    }

    // Check for initial "yakī" followed by consonant
    if initial_four == ['ی', 'ک', 'ی', ' '] && consonants.contains(&hem_reconst[4]) {
        return Some("yaki");
    }

    // Check for initial "chunīn" or "chunān"
    if initial_four == ['چ', 'ن', 'ی', 'ن'] || initial_four == ['چ', 'ن', 'ا', 'ن'] {
        return Some("chunin/an");
    }

    if initial_four == ['چ', 'ی', 'س', 'ت'] {
        return Some("chist");
    }

    if initial_four == ['د', 'و', 'س', 'ت'] {
        return Some("dust");
    }

    // Otherwise we could get tripped up by "nayistān"
    if initial_five == ['ن', 'ی', 'س', 'ت', ' '] {
        return Some("nist");
    }

    if initial_five == ['ه', 'م', 'چ', 'و', ' '] || initial_six == ['ه', 'م', ' ', 'چ', 'و', ' ']
    {
        return Some("hamchu");
    }

    None
}

//
// Results functions
//

fn first_syllable_assessment(
    long_first_syl_markers: u32,
    long_first_syl_locs: String,
    short_first_syl_markers: u32,
    short_first_syl_locs: String,
) -> (bool, bool, String) {
    // Initialize variables for return values
    let mut long_first = false;
    let mut short_first = false;

    let mut first_report = String::from("*** First syllable length ***\n");

    // Report indications of first syllable length
    if long_first_syl_markers > 0 {
        first_report += &format!(
            "Indications of a long first syllable: {} (at {})\n",
            long_first_syl_markers,
            long_first_syl_locs.trim_end_matches(", ")
        );
    }
    if short_first_syl_markers > 0 {
        first_report += &format!(
            "Indications of a short first syllable: {} (at {})\n",
            short_first_syl_markers,
            short_first_syl_locs.trim_end_matches(", ")
        );
    }

    // Report assessment of first syllable length
    if long_first_syl_markers > 0 && short_first_syl_markers > 0 {
        first_report += "There are contradictory indications of a long vs. short first syllable.\n";
        first_report += "If this is not an error, it suggests that the meter is probably ramal.\n";
    } else if long_first_syl_markers > 1 {
        long_first = true;
        first_report += "The first syllable in this meter appears to be long.\n";
    } else if short_first_syl_markers > 1 {
        short_first = true;
        first_report += "The first syllable in this meter appears to be short.\n";
    } else {
        first_report += "So far, insufficient evidence (< 2) of a long vs. short first syllable…\n";
        first_report +=
            "(It's easier to detect short syllables. Scant results may suggest long.)\n";
    }

    (long_first, short_first, first_report)
}

fn second_syllable_assessment(
    long_second_syl_markers: u32,
    long_second_syl_locs: String,
    short_second_syl_markers: u32,
    short_second_syl_locs: String,
) -> (bool, bool, String) {
    // Initialize variables for return values
    let mut long_second = false;
    let mut short_second = false;

    let mut second_report = String::from("*** Second syllable length ***\n");

    // Report indications of second syllable length
    if long_second_syl_markers > 0 {
        second_report += &format!(
            "Suggestions of a long second syllable: {} (at {})\n",
            long_second_syl_markers,
            long_second_syl_locs.trim_end_matches(", ")
        );
        if long_second_syl_markers == 1 {
            second_report += "(Be careful with this; one result is not much.)\n";
        }
    }
    if short_second_syl_markers > 0 {
        second_report += &format!(
            "Suggestions of a short second syllable: {} (at {})\n",
            short_second_syl_markers,
            short_second_syl_locs.trim_end_matches(", ")
        );
        if short_second_syl_markers == 1 {
            second_report += "(Be careful with this; one result is not much.)\n";
        }
    }

    // Report assessment of second syllable length
    if long_second_syl_markers > 0 && short_second_syl_markers > 0 {
        second_report +=
            "There are contradictory indications of a long vs. short second syllable.\n"
    } else if long_second_syl_markers > 1 {
        long_second = true;
        second_report += "The second syllable in this meter appears to be long.\n";
    } else if short_second_syl_markers > 1 {
        short_second = true;
        second_report += "The second syllable in this meter appears to be short.\n"
    } else {
        second_report +=
            "So far, insufficient evidence (< 2) of a long vs. short second syllable…\n"
    }

    (long_second, short_second, second_report)
}

fn final_assessment(
    long_meter: bool,
    short_meter: bool,
    long_first: bool,
    short_first: bool,
    long_second: bool,
    short_second: bool,
) -> String {
    let mut summary_report = String::from("*** Overall assessment ***\n");

    // Long meter
    if long_meter {
        // Long meter, long first syllable
        if long_first {
            // Long meter, long first syllable, short second syllable
            if short_second {
                summary_report += "Long meter, long first syllable, short second syllable?\n";
                summary_report += "Consider ramal.\n";
            // Long meter, long first syllable, long second syllable
            } else if long_second {
                summary_report += "Long meter, long first syllable, long second syllable?\n";
                summary_report +=
                    "Consider, with short third and fourth syllables, hazaj (akhrab).\n";
                summary_report += "Consider, with a long fourth syllable, mużāri‘.\n";
            // Long meter, long first syllable, indeterminate second syllable
            } else {
                summary_report +=
                    "Long meter, long first syllable, indeterminate second syllable?\n";
                summary_report +=
                    "Consider, with a long second syllable, hazaj (akhrab) or mużāri‘.\n";
                summary_report += "Consider, with a short second syllable, ramal.\n";
            }
        // Long meter, short first syllable
        } else if short_first {
            // Long meter, short first syllable, long second syllable
            if long_second {
                summary_report += "Long meter, short first syllable, long second syllable?\n";
                summary_report += "Consider, with a long third syllable, hazaj (sālim).\n";
                summary_report += "Consider, with a short third syllable, mujtaṡṡ.\n";
            // Long meter, short first syllable, short second syllable
            } else if short_second {
                summary_report += "Long meter, short first syllable, short second syllable?\n";
                summary_report += "Consider ramal.\n";
            // Long meter, short first syllable, indeterminate second syllable
            } else {
                summary_report +=
                    "Long meter, short first syllable, indeterminate second syllable?\n";
                summary_report +=
                    "Consider, with a long second syllable, hazaj (sālim) or mujtaṡṡ.\n";
                summary_report += "Consider, with a short second syllable, ramal.\n";
            }
        // Long meter, indeterminate first syllable
        } else {
            summary_report += "What is clearest is that the meter appears to be long.\n";
            summary_report +=
                "If there were mixed signals about the first syllable, consider ramal.\n";
        }
    // Short meter
    } else if short_meter {
        // Short meter, long first syllable
        if long_first {
            // Short meter, long first syllable, short second syllable
            if short_second {
                summary_report += "Short meter, long first syllable, short second syllable?\n";
                summary_report += "Consider ramal or khafīf.\n";
            // Short meter, long first syllable, indeterminate second syllable
            } else {
                summary_report +=
                    "Short meter, long first syllable, indeterminate second syllable?\n";
                summary_report += "Consider, with a long second syllable, hazaj (akhrab).\n";
                summary_report += "Consider, with a short second syllable, ramal or khafīf.\n";
            }
        // Short meter, short first syllable
        } else if short_first {
            // Short meter, short first syllable, short second syllable
            if short_second {
                summary_report += "Short meter, short first syllable, short second syllable?\n";
                summary_report += "This would be rare. Consider ramal.\n";
            // Short meter, short first syllable, indeterminate second syllable
            } else {
                summary_report +=
                    "Short meter, short first syllable, indeterminate second syllable?\n";
                summary_report += "Consider hazaj (musaddas) or mutaqārib (muṡamman).\n";
            }
        // Short meter, indeterminate first syllable
        } else {
            summary_report += "What is clearest is that the meter appears to be short.\n";
            summary_report += "Were there mixed signals about the first syllable?\n";
            summary_report += "If so, consider ramal or khafīf.\n";
        }
    // Indeterminate meter length
    // This currently can't be reached, but I'll leave it just in case
    } else {
        summary_report += "With the meter length unclear, no further conclusions will be drawn.\n";
    }

    summary_report
}
