pub mod fitness;

use std::ops::Deref;

use itertools::iproduct;
use rayon::prelude::*;

use crate::enigma::{Enigma, EnigmaKey, Plugboard, ReflectorId, Rotor, RotorId};
use fitness::FitnessFunction;

pub enum EnigmaAnalysisRotors {
    Three,
    Five,
    Eight,
}

pub struct ScoredEnigmaKey {
    key: EnigmaKey,
    score: f32,
}

impl Deref for ScoredEnigmaKey {
    type Target = EnigmaKey;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl PartialEq for ScoredEnigmaKey {
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}

impl PartialOrd for ScoredEnigmaKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl ScoredEnigmaKey {
    pub fn score(&self) -> f32 {
        self.score
    }
}

pub fn find_rotor_configurations(
    cipher: &str,
    rotors: EnigmaAnalysisRotors,
    plugboard: &[(char, char)],
    required_keys: usize,
    f: &(impl FitnessFunction + Sync),
) -> Vec<ScoredEnigmaKey> {
    let available_rotors: &[RotorId] = match rotors {
        EnigmaAnalysisRotors::Three => &[RotorId::I, RotorId::II, RotorId::III],
        EnigmaAnalysisRotors::Five => &[
            RotorId::I,
            RotorId::II,
            RotorId::III,
            RotorId::IV,
            RotorId::V,
        ],
        EnigmaAnalysisRotors::Eight => &[
            RotorId::I,
            RotorId::II,
            RotorId::III,
            RotorId::IV,
            RotorId::V,
            RotorId::VI,
            RotorId::VII,
            RotorId::VIII,
        ],
    };

    let plugboard = Plugboard::new(plugboard);

    // Collecting ends up being faster as the parallel iterator doesn't need to syncronise access.
    let rotors: Vec<_> = iproduct!(available_rotors, available_rotors, available_rotors)
        .map(|(a, b, c)| (*a, *b, *c))
        .filter(|(a, b, c)| a != b && a != c && b != c)
        .collect();

    let mut key_set: Vec<ScoredEnigmaKey> = rotors
        .into_par_iter() // more cores more better!
        .filter_map(|(a, b, c)| {
            println!("{:?} {:?} {:?}", a, b, c);

            let mut max_fitness: f32 = -1e30;
            let mut best_key = None::<EnigmaKey>;

            const RANGE: std::ops::Range<u8> = 0..26;
            let mut buf = String::with_capacity(cipher.len());
            iproduct!(RANGE, RANGE, RANGE).for_each(|(i, j, k)| {
                let left_rotor = Rotor::new(a, i, 0);
                let middle_rotor = Rotor::new(b, j, 0);
                let right_rotor = Rotor::new(c, k, 0);
                let key = EnigmaKey::new(left_rotor, middle_rotor, right_rotor, plugboard);

                let mut e = Enigma::new(key, ReflectorId::B);

                buf.clear();
                buf.extend(cipher.chars().map(|c| e.encrypt(c)));

                let fitness = f.score(&buf);
                if fitness > max_fitness {
                    max_fitness = fitness;
                    best_key = Some(key);
                }
            });

            best_key.map(|key| ScoredEnigmaKey {
                key,
                score: max_fitness,
            })
        })
        .collect();

    key_set.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse());
    key_set.truncate(required_keys);
    key_set
}

pub fn find_ring_settings(
    cipher: &str,
    mut key: EnigmaKey,
    f: &(impl FitnessFunction + Sync),
) -> ScoredEnigmaKey {
    for &rotor_idx in &[EnigmaKey::right_rotor_mut, EnigmaKey::middle_rotor_mut] {
        let optimal_index = find_ring_setting(key, cipher, rotor_idx, f);
        let rotor = rotor_idx(&mut key);
        rotor.set_ring_setting(optimal_index);
        let rotor_pos = rotor.rotor_position();
        rotor.set_rotor_position((rotor_pos + optimal_index) % 26);
    }

    // Calculate fitness and return scored key.
    let mut enigma = Enigma::new(key, ReflectorId::B);
    let decription: String = cipher.chars().map(|c| enigma.encrypt(c)).collect();
    ScoredEnigmaKey {
        key,
        score: f.score(&decription),
    }
}

fn find_ring_setting(
    mut key: EnigmaKey,
    cipher: &str,
    rotor_idx: fn(&mut EnigmaKey) -> &mut Rotor,
    f: &(impl FitnessFunction + Sync),
) -> u8 {
    let mut optimal_ring_setting = 0;
    let mut max_fitness = -1e30;
    let mut buf = String::with_capacity(cipher.len());

    let start_pos = rotor_idx(&mut key).rotor_position();

    for i in 0..26 {
        let cur_rotor = rotor_idx(&mut key);
        cur_rotor.set_rotor_position((start_pos + i) % 26);
        cur_rotor.set_ring_setting(i);

        let mut enigma = Enigma::new(key, ReflectorId::B);

        buf.clear();
        buf.extend(cipher.chars().map(|c| enigma.encrypt(c)));
        let fitness = f.score(&buf);

        if fitness > max_fitness {
            max_fitness = fitness;
            optimal_ring_setting = i;
        }
    }

    optimal_ring_setting
}

pub fn find_plugs(
    cipher: &str,
    mut key: EnigmaKey,
    max_plugs: u8,
    f: &(impl FitnessFunction + Sync),
) -> ScoredEnigmaKey {
    let mut plugs = Vec::with_capacity(5);

    // We're looking for *up to* max_plugs, we don't have to *have* max_plugs.
    let mut max_fitness = -1e30;
    let mut best_key = key;

    for _ in 0..max_plugs {
        key.set_plugboard(Plugboard::new(&plugs));
        let (fitness, next_plug) = find_plug(key, cipher, f);
        plugs.push(next_plug);

        // The next best plug would make it worse, so stop.
        if fitness < max_fitness {
            break;
        }

        max_fitness = fitness;
        best_key.set_plugboard(Plugboard::new(&plugs));
    }

    let mut enigma = Enigma::new(best_key, ReflectorId::B);
    let decryption: String = cipher.chars().map(|c| enigma.encrypt(c)).collect();
    ScoredEnigmaKey {
        key: best_key,
        score: f.score(&decryption),
    }
}

fn find_plug(
    mut key: EnigmaKey,
    cipher: &str,
    f: &(impl FitnessFunction + Sync),
) -> (f32, (char, char)) {
    let unplugged = key.plugboard().unplugged();
    let mut plugs = key.plugboard().generate_connections();

    let mut optimal_plug = ('A', 'A');
    let mut max_fitness = -1e30;
    let mut buf = String::with_capacity(cipher.len());
    for (_, i) in unplugged.iter().zip(0..).filter(|(v, _)| **v) {
        for (_, j) in unplugged
            .iter()
            .zip(0..)
            .skip(i as usize + 1)
            .filter(|(v, _)| **v)
        {
            let a = (i + b'A') as char;
            let b = (j + b'A') as char;
            let plug = (a, b);

            plugs.push(plug);
            key.set_plugboard(Plugboard::new(&plugs));

            let mut enigma = Enigma::new(key, ReflectorId::B);
            buf.clear();
            buf.extend(cipher.chars().map(|c| enigma.encrypt(c)));

            let fitness = f.score(&buf);
            if fitness > max_fitness {
                max_fitness = fitness;
                optimal_plug = plug;
            }

            plugs.pop(); // Need to make sure we take the tested plug off.
        }
    }

    (max_fitness, optimal_plug)
}
