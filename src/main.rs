#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unnested_or_patterns)]

use anyhow::{Result, anyhow};
use clap::Parser;
use std::fmt::Write as _;
use std::fs;

//
// Types
//

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path of input text file
    #[clap(short, long, value_parser)]
    input: String,
}

#[derive(Debug, Default)]
struct MeterAnalysis {
    long_meter: bool,
    short_meter: bool,
    total_letters: u32,
    syllable_analysis: SyllableAnalysis,
    results_report: String,
}

#[derive(Debug, Default)]
struct SyllableAnalysis {
    long_first_markers: u32,
    long_first_locations: String,
    short_first_markers: u32,
    short_first_locations: String,
    long_second_markers: u32,
    long_second_locations: String,
    short_second_markers: u32,
    short_second_locations: String,
}

impl SyllableAnalysis {
    fn add_long_first(&mut self, hemistich_no: usize) {
        self.long_first_markers += 1;
        self.long_first_locations
            .push_str(&hemistich_no.to_string());
        self.long_first_locations.push_str(", ");
    }

    fn add_short_first(&mut self, hemistich_no: usize) {
        self.short_first_markers += 1;
        self.short_first_locations
            .push_str(&hemistich_no.to_string());
        self.short_first_locations.push_str(", ");
    }

    fn add_long_second(&mut self, hemistich_no: usize) {
        self.long_second_markers += 1;
        self.long_second_locations
            .push_str(&hemistich_no.to_string());
        self.long_second_locations.push_str(", ");
    }

    fn add_short_second(&mut self, hemistich_no: usize) {
        self.short_second_markers += 1;
        self.short_second_locations
            .push_str(&hemistich_no.to_string());
        self.short_second_locations.push_str(", ");
    }
}

//
// Constants
//

const CONSONANTS: [char; 30] = [
    'ء', 'ب', 'پ', 'ت', 'ث', 'ج', 'چ', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'ژ', 'س', 'ش', 'ص', 'ض', 'ط',
    'ظ', 'ع', 'غ', 'ف', 'ق', 'ک', 'گ', 'ل', 'م', 'ن', 'ه',
];

const MAX_FILE_SIZE: u64 = 10_000;
const MIN_HEMISTICHS: usize = 10;
const MAX_HEMISTICHS: usize = 40;

//
// Macros
//

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

//
// Main function
//

fn main() -> Result<()> {
    let args = Args::parse();

    let poem_text = load_poem(&args.input)?;
    let processed_poem = preprocess(&poem_text)?;

    let hemistichs_count = processed_poem.lines().count();
    let mut analysis = analyze_hemistichs(&processed_poem)?;
    analyze_meter_length(&mut analysis, hemistichs_count)?;

    let (long_first, short_first, first_report) = first_syllable_assessment(
        analysis.syllable_analysis.long_first_markers,
        &analysis.syllable_analysis.long_first_locations,
        analysis.syllable_analysis.short_first_markers,
        &analysis.syllable_analysis.short_first_locations,
    )?;
    analysis.results_report += &first_report;

    let (long_second, short_second, second_report) = second_syllable_assessment(
        analysis.syllable_analysis.long_second_markers,
        &analysis.syllable_analysis.long_second_locations,
        analysis.syllable_analysis.short_second_markers,
        &analysis.syllable_analysis.short_second_locations,
    )?;
    analysis.results_report += &second_report;

    let summary_report = final_assessment(
        analysis.long_meter,
        analysis.short_meter,
        long_first,
        short_first,
        long_second,
        short_second,
    );
    analysis.results_report += &summary_report;

    print!("{}", analysis.results_report);
    Ok(())
}

//
// Helper functions
//

fn load_poem(path: &str) -> Result<String> {
    let metadata = fs::metadata(path)
        .map_err(|e| anyhow!("Failed to read file metadata for '{}': {}", path, e))?;

    let file_size = metadata.len();
    if file_size > MAX_FILE_SIZE {
        return Err(anyhow!(
            "File '{}' is too large ({} bytes). Maximum allowed size is {} bytes.",
            path,
            file_size,
            MAX_FILE_SIZE
        ));
    }

    fs::read_to_string(path).map_err(|e| anyhow!("Failed to read file '{}': {}", path, e))
}

fn preprocess(poem: &str) -> Result<String> {
    let re = regex!(r"\n{2,}");
    let trimmed = re.replace_all(poem.trim(), "\n");

    let line_count = trimmed.lines().count();
    if line_count < MIN_HEMISTICHS {
        return Err(anyhow!(
            "Poem is too short. Found {} hemistichs; at least {} are required.",
            line_count,
            MIN_HEMISTICHS
        ));
    }

    Ok(trimmed.into_owned())
}

fn analyze_hemistichs(poem_text: &str) -> Result<MeterAnalysis> {
    let mut analysis = MeterAnalysis {
        results_report: String::from("*** Assessing the following hemistichs ***\n"),
        ..Default::default()
    };

    let hemistichs: Vec<&str> = poem_text.lines().collect();

    for (i, &hem) in hemistichs.iter().enumerate().take(MAX_HEMISTICHS) {
        let hem_no = i + 1;

        let hem_reconst = reconstruct_hemistich(hem)?;
        let mut hem_nospace = hem_reconst.clone();
        hem_nospace.retain(|x| *x != ' ');

        let hem_reconst_str: String = hem_reconst.iter().collect();
        writeln!(analysis.results_report, "{hem_no}: {hem_reconst_str}")?;

        #[allow(clippy::cast_possible_truncation)]
        let hem_letter_count = hem_nospace.len() as u32;
        analysis.total_letters += hem_letter_count;

        analyze_syllables(
            &hem_reconst,
            &hem_nospace,
            hem_no,
            &mut analysis.syllable_analysis,
        );
    }

    Ok(analysis)
}

fn analyze_syllables(
    hem_reconst: &[char],
    hem_nospace: &[char],
    hem_no: usize,
    analysis: &mut SyllableAnalysis,
) {
    if long_first_syllable(hem_reconst) {
        analysis.add_long_first(hem_no);
    }

    if short_first_syllable(hem_reconst) {
        analysis.add_short_first(hem_no);
    }

    if long_second_syllable(hem_reconst) {
        analysis.add_long_second(hem_no);
    }

    if short_second_syllable(hem_reconst, hem_nospace) {
        analysis.add_short_second(hem_no);
    }

    if let Some(result) = initial_clues(hem_reconst) {
        match result {
            "kasi" | "yaki" => {
                analysis.add_short_first(hem_no);
                analysis.add_long_second(hem_no);
            }
            "chist" | "dust" | "nist" | "ham-chu" | "kist" => {
                analysis.add_long_first(hem_no);
                analysis.add_short_second(hem_no);
            }
            "chandan" => {
                analysis.add_long_first(hem_no);
                analysis.add_long_second(hem_no);
            }
            _ => {}
        }
    }
}

fn analyze_meter_length(analysis: &mut MeterAnalysis, total_hemistichs: usize) -> Result<()> {
    let total_letters_f = f64::from(analysis.total_letters);

    #[allow(clippy::cast_precision_loss)]
    let total_hemistichs_f = if total_hemistichs > MAX_HEMISTICHS {
        MAX_HEMISTICHS as f64
    } else {
        total_hemistichs as f64
    };

    let avg_letters = total_letters_f / total_hemistichs_f;

    analysis.results_report += "*** Meter length ***\n";
    writeln!(
        analysis.results_report,
        "Average letters per hemistich: {avg_letters:.1}"
    )?;

    if avg_letters >= 23.5 {
        analysis.long_meter = true;
        analysis.results_report += "The meter appears to be long (muṡamman).\n";
    } else if avg_letters >= 22.5 {
        analysis.long_meter = true;
        analysis.results_report += "The meter appears to be long (muṡamman).\n";
        analysis.results_report += "(But this is pretty short for a long meter!)\n";
    } else if avg_letters >= 21.0 {
        analysis.short_meter = true;
        analysis.results_report +=
            "The meter appears to be short (musaddas; or mutaqārib muṡamman).\n";
        analysis.results_report += "(But this is pretty long for a short meter!)\n";
    } else {
        analysis.short_meter = true;
        analysis.results_report +=
            "The meter appears to be short (musaddas; or mutaqārib muṡamman).\n";
    }

    Ok(())
}

//
// Analysis functions
//

fn reconstruct_hemistich(hem: &str) -> Result<Vec<char>> {
    // Create a vec for reconstruction
    let mut hem_reconst = Vec::new();

    // Review one character at a time, passing through valid input
    for c in hem.trim().chars() {
        #[allow(clippy::match_same_arms)]
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
            // tanwīn fatḥah, dagger alif, tanwīn kasrah, tanwīn ḍammah
            'ٔ' | 'َ' | 'ّ' | 'ُ' | 'ِ' | 'ْ' | 'ً' | 'ٰ' | 'ٍ' | 'ٌ' => {}
            // Spaces can stay (for now)
            ' ' => hem_reconst.push(c),
            // ZWNJ becomes space
            '‌' => hem_reconst.push(' '),
            // Ignore comma, question mark, or exclamation mark
            '،' | '؟' | '!' => {}

            // Flag anything else
            _ => {
                return Err(anyhow!(
                    "Unexpected character: {}. Text must be fully in Persian/Arabic script.",
                    c.escape_unicode()
                ));
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

    // This would panic if hem_reconst.len() < 5, but I've never seen that
    let initial_three = &hem_reconst[0..3];
    let initial_five = &hem_reconst[0..5];

    // Check for initial "īn" or "khwā-"
    // I found one word that would break this: "khavāniq"
    // But that's vanishingly rare -- only one poem on Ganjoor has it at all,
    // and not at the start of a hemistich
    if matches!(initial_three, ['ا', 'ی', 'ن'] | ['خ', 'و', 'ا']) {
        return true;
    }

    // Check for initial "az," "har," "gar," "ay," or "ham" followed by a space
    // and then a consonant
    // Used to check here for "bar," but it caused a problem -- it can be
    // "bar-i" with iżāfa
    if matches!(
        initial_three,
        ['ا', 'ز', ' '] | ['ه', 'ر', ' '] | ['گ', 'ر', ' '] | ['ا', 'ی', ' '] | ['ه', 'م', ' ']
    ) && CONSONANTS.contains(&hem_reconst[3])
    {
        return true;
    }

    // Check for initial "amrūz"
    // This will also have been flagged for a long second syllable
    if matches!(initial_five, ['ا', 'م', 'ر', 'و', 'ز']) {
        return true;
    }

    false
}

fn short_first_syllable(hem_reconst: &[char]) -> bool {
    // Check for initial "zih" followed by a consonant (after a space)
    if hem_reconst[0..2] == ['ز', ' '] && CONSONANTS.contains(&hem_reconst[2]) {
        return true;
    }

    // Check first three characters
    // Initial "bi" (risky?), "ki," "chu," "chi," or "na" (risky?) followed
    // by a space
    // Initial "kujā," "hamī," "khudā," "agar," "chirā," or "digar," with or
    // without a space
    if matches!(
        hem_reconst[0..3],
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
            | ['د', 'گ', 'ر']
    ) {
        return true;
    }

    // Check first four characters
    // Initial "shavad," "magar," "marā,"" "turā," or "hama" followed by a
    // space; or initial "chunīn" or "chunān" or "bi-bīn-," with or without a
    // space
    if matches!(
        hem_reconst[0..4],
        ['ش', 'و', 'د', ' ']
            | ['م', 'گ', 'ر', ' ']
            | ['م', 'ر', 'ا', ' ']
            | ['ت', 'ر', 'ا', ' ']
            | ['ه', 'م', 'ه', ' ']
            | ['چ', 'ن', 'ی', 'ن']
            | ['چ', 'ن', 'ا', 'ن']
            | ['ب', 'ب', 'ی', 'ن']
    ) {
        return true;
    }

    false
}

fn long_second_syllable(hem_reconst: &[char]) -> bool {
    let second = hem_reconst[1];

    let initial_three = &hem_reconst[0..3];
    let initial_four = &hem_reconst[0..4];
    let initial_five = &hem_reconst[0..5];

    // Check for alif as third character, non-word-initial, not after vāv
    // Also need to make sure the preceding character isn't another alif
    // This caused a problem with "nā-umīd" -- second syllable is short!
    // Should maybe work on better criteria for alif qua long vowel marker
    if hem_reconst[2] == 'ا' && !matches!(second, ' ' | 'و' | 'ا') {
        return true;
    }

    // Check for initial "agar" followed by a consonant
    // This would already have been flagged for a short first syllable
    if initial_four == ['ا', 'گ', 'ر', ' '] && CONSONANTS.contains(&hem_reconst[4]) {
        return true;
    }

    // Check for initial "bāshad" followed by a consonant
    // This would already have been flagged for a long first syllable
    // Used to check here for initial "sāqī," but that can be spoiled by iżāfa
    if initial_five == ['ب', 'ا', 'ش', 'د', ' '] && CONSONANTS.contains(&hem_reconst[5]) {
        return true;
    }

    // Check for initial "amrūz"
    // This will also have been flagged for a long first syllable
    if initial_five == ['ا', 'م', 'ر', 'و', 'ز'] {
        return true;
    }

    // If the opening word is anything like "tā," "bā," "yā," etc., check if
    // what follows is clearly another long syllable
    if hem_reconst[1..3] == ['ا', ' '] && long_first_syllable(&hem_reconst[3..]) {
        return true;
    }

    // If the opening word is "ay," "gar," or "az," followed by a consonant,
    // check if what follows is clearly another long syllable
    if matches!(
        initial_three,
        ['ا', 'ی', ' '] | ['گ', 'ر', ' '] | ['ا', 'ز', ' ']
    ) && CONSONANTS.contains(&hem_reconst[3])
        && long_first_syllable(&hem_reconst[3..])
    {
        return true;
    }

    // If the opening word is "bi" or "ki" (short), check if what follows is
    // clearly a long syllable
    // Is this legit? It's worth a shot
    if matches!(initial_three, ['ب', 'ه', ' '] | ['ک', 'ه', ' '])
        && long_first_syllable(&hem_reconst[3..])
    {
        return true;
    }

    // Check for initial "chunīn" or "chunān," with or without a space
    // This will also have been flagged for a short first syllable
    if matches!(initial_four, ['چ', 'ن', 'ی', 'ن'] | ['چ', 'ن', 'ا', 'ن']) {
        return true;
    }

    false
}

fn short_second_syllable(hem_reconst: &[char], hem_nospace: &[char]) -> bool {
    let initial_three = &hem_reconst[0..3];
    let initial_four = &hem_reconst[0..4];
    let initial_five = &hem_reconst[0..5];
    let initial_six = &hem_reconst[0..6];

    // If the opening word is "bi" or "ki" (very common), check if what
    // follows is clearly another short syllable
    if matches!(initial_three, ['ب', 'ه', ' '] | ['ک', 'ه', ' '])
        && short_first_syllable(&hem_reconst[3..])
    {
        return true;
    }

    // If the opening word is anything like "tā," "bā," "yā," etc., check if
    // what follows is clearly a short syllable
    if hem_reconst[1..3] == ['ا', ' '] && short_first_syllable(&hem_reconst[3..]) {
        return true;
    }

    // Some of the below imply a long first syllable that would not have been
    // caught otherwise. Such cases should be dealt with instead in "initial
    // clues"

    // Check for initial "har-ki," "ān-ki," "gar-chi," or "ān-chi" (with or
    // without a space)
    // "Gar-chi" has now caused a problem -- "chi" can be long? Should I get
    // rid of it? But this seems very rare

    // Also check for initial "pādishā-"
    // This will already have been flagged for a long first syllable

    if matches!(
        initial_five,
        ['ه', 'ر', 'ک', 'ه', ' ']
            | ['آ', 'ن', 'ک', 'ه', ' ']
            | ['گ', 'ر', 'چ', 'ه', ' ']
            | ['آ', 'ن', 'چ', 'ه', ' ']
            | ['پ', 'ا', 'د', 'ش', 'ا']
    ) {
        return true;
    }

    if matches!(
        initial_six,
        ['ه', 'ر', ' ', 'ک', 'ه', ' ']
            | ['آ', 'ن', ' ', 'ک', 'ه', ' ']
            | ['گ', 'ر', ' ', 'چ', 'ه', ' ']
            | ['آ', 'ن', ' ', 'چ', 'ه', ' ']
    ) {
        return true;
    }

    // Used to check here for near-initial "kunad" or "shavad"
    // Could try to bring that back somehow?

    let two_six = &hem_nospace[2..6];

    // Check for "chunīn" or "chunān" starting at the third letter (with or
    // without a space). I think this is valid
    // But I may get rid of this approach. I don't like it somehow
    if matches!(two_six, ['چ', 'ن', 'ی', 'ن'] | ['چ', 'ن', 'ا', 'ن']) {
        return true;
    }

    // If the opening word is "īn," followed by a space and then a consonant,
    // check if what follows is clearly a short syllable
    if initial_four == ['ا', 'ی', 'ن', ' ']
        && CONSONANTS.contains(&hem_reconst[4])
        && short_first_syllable(&hem_reconst[4..])
    {
        return true;
    }

    false
}

fn initial_clues(hem_reconst: &[char]) -> Option<&str> {
    let initial_four = &hem_reconst[0..4];
    let initial_five = &hem_reconst[0..5];
    let initial_six = &hem_reconst[0..6];

    // Check for initial "kasī" followed by a consonant
    if initial_four == ['ک', 'س', 'ی', ' '] && CONSONANTS.contains(&hem_reconst[4]) {
        return Some("kasi");
    }

    // Check for initial "yakī" followed by a consonant
    if initial_four == ['ی', 'ک', 'ی', ' '] && CONSONANTS.contains(&hem_reconst[4]) {
        return Some("yaki");
    }

    // Check for initial "chīst"
    // This should always scan long-short, regardless of what follows
    if initial_four == ['چ', 'ی', 'س', 'ت'] {
        return Some("chist");
    }

    // Check for initial "dūst"
    // This should always scan long-short, regardless of what follows
    if initial_four == ['د', 'و', 'س', 'ت'] {
        return Some("dust");
    }

    // Check for initial "nīst" followed by a space
    // This should scan long-short
    // Without the space, we could get tripped up by "nayistān"
    if initial_five == ['ن', 'ی', 'س', 'ت', ' '] {
        return Some("nist");
    }

    // Check for initial "ham-chu" followed by a space (with or without an
    // internal space)
    if initial_five == ['ه', 'م', 'چ', 'و', ' '] || initial_six == ['ه', 'م', ' ', 'چ', 'و', ' ']
    {
        return Some("ham-chu");
    }

    // Check for initial "chandān"
    // This should always scan long-long, regardless of what follows
    if initial_five == ['چ', 'ن', 'د', 'ا', 'ن'] {
        return Some("chandan");
    }

    // Check for initial "kīst"
    // This should always scan long-short, regardless of what follows
    if initial_four == ['ک', 'ی', 'س', 'ت'] {
        return Some("kist");
    }

    None
}

//
// Results functions
//

fn first_syllable_assessment(
    long_first_syl_markers: u32,
    long_first_syl_locs: &str,
    short_first_syl_markers: u32,
    short_first_syl_locs: &str,
) -> Result<(bool, bool, String)> {
    // Initialize variables for return values
    let mut long_first = false;
    let mut short_first = false;

    let mut first_report = String::from("*** First syllable length ***\n");

    // Report indications of first syllable length
    if long_first_syl_markers > 0 {
        writeln!(
            first_report,
            "Indications of a long first syllable: {} (at {})",
            long_first_syl_markers,
            long_first_syl_locs.trim_end_matches(", ")
        )?;
    }
    if short_first_syl_markers > 0 {
        writeln!(
            first_report,
            "Indications of a short first syllable: {} (at {})",
            short_first_syl_markers,
            short_first_syl_locs.trim_end_matches(", ")
        )?;
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
        first_report += "Insufficient evidence (< 2) of a long vs. short first syllable…\n";
        first_report +=
            "(It's easier to detect short syllables. Scant results may suggest long.)\n";
    }

    Ok((long_first, short_first, first_report))
}

fn second_syllable_assessment(
    long_second_syl_markers: u32,
    long_second_syl_locs: &str,
    short_second_syl_markers: u32,
    short_second_syl_locs: &str,
) -> Result<(bool, bool, String)> {
    // Initialize variables for return values
    let mut long_second = false;
    let mut short_second = false;

    let mut second_report = String::from("*** Second syllable length ***\n");

    // Report indications of second syllable length
    if long_second_syl_markers > 0 {
        writeln!(
            second_report,
            "Suggestions of a long second syllable: {} (at {})",
            long_second_syl_markers,
            long_second_syl_locs.trim_end_matches(", ")
        )?;
        if long_second_syl_markers == 1 {
            second_report += "(Be careful with this; one result is not much.)\n";
        }
    }
    if short_second_syl_markers > 0 {
        writeln!(
            second_report,
            "Suggestions of a short second syllable: {} (at {})",
            short_second_syl_markers,
            short_second_syl_locs.trim_end_matches(", ")
        )?;
        if short_second_syl_markers == 1 {
            second_report += "(Be careful with this; one result is not much.)\n";
        }
    }

    // Report assessment of second syllable length
    if long_second_syl_markers > 0 && short_second_syl_markers > 0 {
        second_report +=
            "There are contradictory indications of a long vs. short second syllable.\n";
    } else if long_second_syl_markers > 1 {
        long_second = true;
        second_report += "The second syllable in this meter appears to be long.\n";
    } else if short_second_syl_markers > 1 {
        short_second = true;
        second_report += "The second syllable in this meter appears to be short.\n";
    } else {
        second_report += "Insufficient evidence (< 2) of a long vs. short second syllable…\n";
    }

    Ok((long_second, short_second, second_report))
}

#[allow(clippy::fn_params_excessive_bools)]
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
            // Long meter, long first syllable, long second syllable
            if long_second {
                summary_report += "Long meter, long first syllable, long second syllable?\n";
                summary_report +=
                    "Consider, with short third and fourth syllables, hazaj (akhrab).\n";
                summary_report += "Consider, with a long fourth syllable, mużāri‘.\n";
            // Long meter, long first syllable, short second syllable
            } else if short_second {
                summary_report += "Long meter, long first syllable, short second syllable?\n";
                summary_report += "Consider ramal.\n";
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
            // Short meter, long first syllable, long second syllable
            if long_second {
                summary_report += "Short meter, long first syllable, long second syllable?\n";
                summary_report += "Consider hazaj (akhrab).\n";
            // Short meter, long first syllable, short second syllable
            } else if short_second {
                summary_report += "Short meter, long first syllable, short second syllable?\n";
                summary_report += "Consider, with a long third syllable, ramal or khafīf.\n";
                summary_report += "If the third syllable is short, enjoy the puzzle!\n";
            // Short meter, long first syllable, indeterminate second syllable
            } else {
                summary_report +=
                    "Short meter, long first syllable, indeterminate second syllable?\n";
                summary_report += "Consider, with a long second syllable, hazaj (akhrab).\n";
                summary_report += "Consider, with a short second syllable, ramal or khafīf.\n";
            }
        // Short meter, short first syllable
        } else if short_first {
            // Short meter, short first syllable, long second syllable
            if long_second {
                summary_report += "Short meter, short first syllable, long second syllable?\n";
                summary_report += "Consider hazaj or mutaqārib.\n";
            // Short meter, short first syllable, short second syllable
            } else if short_second {
                summary_report += "Short meter, short first syllable, short second syllable?\n";
                summary_report += "This would be rare. Consider ramal or khafīf.\n";
            // Short meter, short first syllable, indeterminate second syllable
            } else {
                summary_report +=
                    "Short meter, short first syllable, indeterminate second syllable?\n";
                summary_report += "Consider, with a long second syllable, hazaj or mutaqārib.\n";
                summary_report += "Consider, with a short second syllable, ramal or khafīf.\n";
            }
        // Short meter, indeterminate first syllable
        } else {
            summary_report += "What is clearest is that the meter appears to be short.\n";
            summary_report += "Were there mixed signals about the first syllable?\n";
            summary_report += "If so, consider ramal or khafīf.\n";
        }
    // Indeterminate meter length
    // This currently can't be reached; I'll leave it for possible future use
    } else {
        summary_report += "With the meter length unclear, no further conclusions will be drawn.\n";
    }

    summary_report
}
