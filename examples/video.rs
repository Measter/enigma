use std::time::Instant;

use enigma::{
    analysis::{
        find_plugs, find_ring_settings, find_rotor_configurations,
        fitness::{IoCFitness, NgramFitness},
        EnigmaAnalysisRotors,
    },
    enigma::{Enigma, ReflectorId},
};

const SINGLE: &str = include_str!("../data/single");
const BIGRAMS: &str = include_str!("../data/bigrams");
const TRIGRAMS: &str = include_str!("../data/trigrams");
const QUADGRAMS: &str = include_str!("../data/quadgrams");

// For those interested, these were the original settings
// II V III / 7 4 19 / 12 2 20 / AF TV KO BL RW
const CIPHER_TEXT: &str = "OZLUDYAKMGMXVFVARPMJIKVWPMBVWMOIDHYPLAYUWGBZFAFAFUQFZQISLEZMYPVBRDDLAGIHIFUJDFADORQOOMIZP\
                           YXDCBPWDSSNUSYZTJEWZPWFBWBMIEQXRFASZLOPPZRJKJSPPSTXKPUWYSKNMZZLHJDXJMMMDFODIHUBVCXMNICNYQ\
                           BNQODFQLOGPZYXRJMTLMRKQAUQJPADHDZPFIKTQBFXAYMVSZPKXIQLOQCVRPKOBZSXIUBAAJBRSNAFDMLLBVSYXIS\
                           FXQZKQJRIQHOSHVYJXIFUZRMXWJVWHCCYHCXYGRKMKBPWRDBXXRGABQBZRJDVHFPJZUSEBHWAEOGEUQFZEEBDCWND\
                           HIAQDMHKPRVYHQGRDYQIOEOLUBGBSNXWPZCHLDZQBWBEWOCQDBAFGUVHNGCIKXEIZGIZHPJFCTMNNNAUXEVWTWACH\
                           OLOLSLTMDRZJZEVKKSSGUUTHVXXODSKTFGRUEIIXVWQYUIPIDBFPGLBYXZTCOQBCAHJYNSGDYLREYBRAKXGKQKWJE\
                           KWGAPTHGOMXJDSQKYHMFGOLXBSKVLGNZOAXGVTGXUIVFTGKPJU";

fn main() {
    let ioc = IoCFitness::new();
    let bigrams = NgramFitness::<2>::new(BIGRAMS.lines());
    let quadgrams = NgramFitness::<4>::new(QUADGRAMS.lines());

    let start_time = Instant::now();

    let rotor_configurations =
        find_rotor_configurations(CIPHER_TEXT, EnigmaAnalysisRotors::Five, &[], 10, &ioc);

    println!("Rotor search time: {:?}", start_time.elapsed());

    println!("\nTop 10 rotor configurations:");
    for key in &rotor_configurations {
        println!("{}", **key);
    }

    let mut enigma = Enigma::new(*rotor_configurations[0], ReflectorId::B);
    let output: String = CIPHER_TEXT.chars().map(|c| enigma.encrypt(c)).collect();
    println!("Current decryption: {}\n", output);

    // Next find the best ring settings for the best configuration (index 0);
    let rotor_and_ring_configuration =
        find_ring_settings(CIPHER_TEXT, *rotor_configurations[0], &bigrams);

    println!("{}", *rotor_and_ring_configuration);

    let mut enigma = Enigma::new(*rotor_and_ring_configuration, ReflectorId::B);
    let output: String = CIPHER_TEXT.chars().map(|c| enigma.encrypt(c)).collect();
    println!("Current decryption: {}\n", output);

    // Finally, perform hill climbing to find plugs one at a time.
    let optimal_key_with_plugs =
        find_plugs(CIPHER_TEXT, *rotor_and_ring_configuration, 10, &quadgrams);
    println!("{}", *optimal_key_with_plugs);

    let mut enigma = Enigma::new(*optimal_key_with_plugs, ReflectorId::B);
    let output: String = CIPHER_TEXT.chars().map(|c| enigma.encrypt(c)).collect();
    println!("Final decryption: {}", output);

    println!("\nTotal execution time: {:?}", start_time.elapsed());
}
