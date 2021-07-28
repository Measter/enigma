use std::fmt::{Display, Write};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RotorId {
    I = 0,
    II = 1,
    III = 2,
    IV = 3,
    V = 4,
    VI = 5,
    VII = 6,
    VIII = 7,

    Identity = 8,
}

// Because the rotor wiring is a fixed value, I decided to generate these at compile time. The Java version
// generates them at runtime, but uses fixed values, so the end result never changes.
// But, if you did want to have these be a run-time input, the way you'd do it would be to generate them
// once at program startup, then you could have RotorID be a struct that holds the notch positions and a
// reference to the mapping arrays and returns them in the forward_wiring and reverse_wiring functions.
const ROTOR_FORWARD_WIRING: [[u8; 26]; 9] = [
    RotorId::gen_forward_wiring(RotorId::I),
    RotorId::gen_forward_wiring(RotorId::II),
    RotorId::gen_forward_wiring(RotorId::III),
    RotorId::gen_forward_wiring(RotorId::IV),
    RotorId::gen_forward_wiring(RotorId::V),
    RotorId::gen_forward_wiring(RotorId::VI),
    RotorId::gen_forward_wiring(RotorId::VII),
    RotorId::gen_forward_wiring(RotorId::VIII),
    RotorId::gen_forward_wiring(RotorId::Identity),
];

const ROTOR_BACKWARD_WIRING: [[u8; 26]; 9] = [
    RotorId::gen_backward_wiring(RotorId::I),
    RotorId::gen_backward_wiring(RotorId::II),
    RotorId::gen_backward_wiring(RotorId::III),
    RotorId::gen_backward_wiring(RotorId::IV),
    RotorId::gen_backward_wiring(RotorId::V),
    RotorId::gen_backward_wiring(RotorId::VI),
    RotorId::gen_backward_wiring(RotorId::VII),
    RotorId::gen_backward_wiring(RotorId::VIII),
    RotorId::gen_backward_wiring(RotorId::Identity),
];

impl RotorId {
    fn is_at_notch(self, position: u8) -> bool {
        match self {
            RotorId::I => position == 16,
            RotorId::II => position == 4,
            RotorId::III => position == 21,
            RotorId::IV => position == 9,
            RotorId::V => position == 25,
            RotorId::VI => position == 12 || position == 25,
            RotorId::VII => position == 12 || position == 25,
            RotorId::VIII => position == 12 || position == 25,
            RotorId::Identity => position == 0,
        }
    }

    const fn chars(s: Self) -> [u8; 26] {
        *match s {
            RotorId::I => b"EKMFLGDQVZNTOWYHXUSPAIBRCJ",
            RotorId::II => b"AJDKSIRUXBLHWTMCQGZNPYFVOE",
            RotorId::III => b"BDFHJLCPRTXVZNYEIWGAKMUSQO",
            RotorId::IV => b"ESOVPZJAYQUIRHXLNFTGKDCMWB",
            RotorId::V => b"VZBRGITYUPSDNHLXAWMJQOFECK",
            RotorId::VI => b"JPGVOUMFYQBENHZRDKASXLICTW",
            RotorId::VII => b"NZJHGRCXMYSWBOUFAIVLPEKQDT",
            RotorId::VIII => b"FKQHTLXOCBJSPDZRAMEWNIUYGV",
            RotorId::Identity => b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        }
    }

    const fn gen_forward_wiring(s: Self) -> [u8; 26] {
        let chars = Self::chars(s);
        let mut wiring = [0; 26];

        let mut i = 0;
        while i < 26 {
            wiring[i] = chars[i] - b'A';
            i += 1;
        }

        wiring
    }

    const fn gen_backward_wiring(s: Self) -> [u8; 26] {
        let forward_wiring = Self::gen_forward_wiring(s);
        let mut backwards_wiring = [0; 26];

        let mut i = 0;
        while i < 26 {
            backwards_wiring[forward_wiring[i] as usize] = i as u8;
            i += 1;
        }

        backwards_wiring
    }

    fn forward_wiring(self) -> &'static [u8; 26] {
        &ROTOR_FORWARD_WIRING[self as usize]
    }

    fn backward_wiring(self) -> &'static [u8; 26] {
        &ROTOR_BACKWARD_WIRING[self as usize]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReflectorId {
    B = 0,
    C = 1,
    Default = 2,
}

const REFLECTOR_WIRING: [[u8; 26]; 3] = [
    ReflectorId::gen_wiring(ReflectorId::B),
    ReflectorId::gen_wiring(ReflectorId::C),
    ReflectorId::gen_wiring(ReflectorId::Default),
];

impl ReflectorId {
    const fn gen_wiring(self) -> [u8; 26] {
        let mut wiring = *match self {
            ReflectorId::B => b"YRUHQSLDPXNGOKMIEBFZCWVJAT",
            ReflectorId::C => b"FVPJIAOYEDRZXWGCTKUQSBNMHL",
            ReflectorId::Default => b"ZYXWVUTSRQPONMLKJIHGFEDCBA",
        };

        let mut i = 0;
        while i < 26 {
            wiring[i] -= b'A';
            i += 1;
        }
        wiring
    }

    fn forward(self, c: u8) -> u8 {
        REFLECTOR_WIRING[self as usize][c as usize]
    }
}

// Because the Rotors are just a couple numbers, this ends up being massively cheaper to create
// than in the Java version, which re-parses the rotor wiring each time.
// The Plugboard is still parsed at runtime, but the type is only 26 bytes, so is cheap to copy.
#[derive(Debug, Clone, Copy)]
pub struct EnigmaKey {
    left_rotor: Rotor,
    middle_rotor: Rotor,
    right_rotor: Rotor,
    plugboard: Plugboard,
}

impl EnigmaKey {
    pub fn new(
        left_rotor: Rotor,
        middle_rotor: Rotor,
        right_rotor: Rotor,
        plugboard: Plugboard,
    ) -> Self {
        Self {
            left_rotor,
            middle_rotor,
            right_rotor,
            plugboard,
        }
    }

    /// Get a reference to the enigma key's left rotor.
    pub fn left_rotor(&self) -> &Rotor {
        &self.left_rotor
    }

    /// Get a reference to the enigma key's middle rotor.
    pub fn middle_rotor(&self) -> &Rotor {
        &self.middle_rotor
    }

    /// Get a reference to the enigma key's right rotor.
    pub fn right_rotor(&self) -> &Rotor {
        &self.right_rotor
    }

    /// Get a mutable reference to the enigma key's left rotor.
    pub fn left_rotor_mut(&mut self) -> &mut Rotor {
        &mut self.left_rotor
    }

    /// Get a mutable reference to the enigma key's middle rotor.
    pub fn middle_rotor_mut(&mut self) -> &mut Rotor {
        &mut self.middle_rotor
    }

    /// Get a mutable reference to the enigma key's right rotor.
    pub fn right_rotor_mut(&mut self) -> &mut Rotor {
        &mut self.right_rotor
    }

    /// Get a reference to the enigma key's plugboard.
    pub fn plugboard(&self) -> &Plugboard {
        &self.plugboard
    }

    /// Set the enigma key's plugboard.
    pub fn set_plugboard(&mut self, plugboard: Plugboard) {
        self.plugboard = plugboard;
    }
}

impl Display for EnigmaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key")
            .field(
                "Left Rotor",
                &format_args!(
                    "{:?} {} {}",
                    self.left_rotor.id,
                    self.left_rotor.rotor_position,
                    self.left_rotor.ring_setting
                ),
            )
            .field(
                "Middle Rotor",
                &format_args!(
                    "{:?} {} {}",
                    self.middle_rotor.id,
                    self.middle_rotor.rotor_position,
                    self.middle_rotor.ring_setting
                ),
            )
            .field(
                "Right Rotor",
                &format_args!(
                    "{:?} {} {}",
                    self.right_rotor.id,
                    self.right_rotor.rotor_position,
                    self.right_rotor.ring_setting
                ),
            )
            .field("Plugboard", &format_args!("{}", self.plugboard))
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rotor {
    id: RotorId,
    rotor_position: u8,
    ring_setting: u8,
}

impl Rotor {
    pub fn new(id: RotorId, rotor_position: u8, ring_setting: u8) -> Self {
        assert!((0..26).contains(&rotor_position));
        assert!((0..26).contains(&ring_setting));

        Self {
            id,
            rotor_position,
            ring_setting,
        }
    }

    fn is_at_notch(&self) -> bool {
        self.id.is_at_notch(self.rotor_position)
    }

    fn turnover(&mut self) {
        self.rotor_position = match self.rotor_position + 1 {
            v @ 0..=25 => v,
            v => v - 26,
        };
    }

    // This is the hottest of hot functions. In the video example, I calculated it gets run somewhere
    // in the region of 500 million times.
    /// Requires that `c`, `ring` and `pos` are in the range 0..26.
    fn encypher(c: u8, pos: u8, ring: u8, mapping: &[u8; 26]) -> u8 {
        // let shift = pos - ring;
        // let idx = (c + shift + 26) % 26;
        // let val = (mapping[idx as usize] - shift + 26) % 26;
        // val

        // The following recreates the logic from above for the specific inputs we have.
        // The two modulo instructions have a fairly high cost, and this is the hottest
        // of hot functions in this program.
        let shift = match pos.overflowing_sub(ring) {
            (x, true) => x + 26,
            (x, false) => x,
        };
        let idx = match c + shift {
            v @ 0..=25 => v,
            v => v - 26,
        };

        let val = mapping[idx as usize];
        match val.overflowing_sub(shift) {
            (x, true) => x + 26,
            (x, false) => x,
        }
    }

    /// Assumes that `c` is in the range 0..26.
    fn forward(&self, c: u8) -> u8 {
        Self::encypher(
            c,
            self.rotor_position,
            self.ring_setting,
            self.id.forward_wiring(),
        )
    }

    /// Assumes that `c` is in the range 0..26.
    fn backward(&self, c: u8) -> u8 {
        Self::encypher(
            c,
            self.rotor_position,
            self.ring_setting,
            self.id.backward_wiring(),
        )
    }

    /// Get a reference to the rotor's id.
    pub fn id(&self) -> &RotorId {
        &self.id
    }

    /// Get a reference to the rotor's rotor position.
    pub fn rotor_position(&self) -> u8 {
        self.rotor_position
    }

    /// Get a reference to the rotor's ring setting.
    pub fn ring_setting(&self) -> u8 {
        self.ring_setting
    }

    /// Set the rotor's rotor position.
    pub fn set_rotor_position(&mut self, rotor_position: u8) {
        assert!((0..26).contains(&rotor_position));
        self.rotor_position = rotor_position;
    }

    /// Set the rotor's ring setting.
    pub fn set_ring_setting(&mut self, ring_setting: u8) {
        assert!((0..26).contains(&ring_setting));
        self.ring_setting = ring_setting;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Plugboard {
    wiring: [u8; 26],
}

impl Plugboard {
    pub fn new(connections: &[(char, char)]) -> Self {
        Self {
            wiring: Self::decode_plugboard(connections),
        }
    }

    fn identity() -> [u8; 26] {
        let mut wiring = [0; 26];
        wiring.iter_mut().zip(0..).for_each(|(w, i)| *w = i);

        wiring
    }

    fn decode_plugboard(connections: &[(char, char)]) -> [u8; 26] {
        let mut mapping = Self::identity();

        if connections.is_empty() {
            return mapping;
        }

        // No need for fancy hashsets, we're doing ASCII!
        let mut seen = [false; 26];

        for (rc1, rc2) in connections {
            if !rc1.is_ascii_uppercase() || !rc2.is_ascii_uppercase() {
                panic!(
                    "Plugboard init error: Invalid character pair: ({:?}, {:?})",
                    rc1, rc2
                );
            }

            let c1 = *rc1 as u8 - b'A';
            let c2 = *rc2 as u8 - b'A';

            if seen[c1 as usize] || seen[c2 as usize] {
                panic!(
                    "Plugboard init error: Duplicate plug: ({:?}, {:?})",
                    rc1, rc2
                );
            }

            seen[c1 as usize] = true;
            seen[c2 as usize] = true;

            mapping[c1 as usize] = c2;
            mapping[c2 as usize] = c1;
        }

        mapping
    }

    fn forward(&self, c: u8) -> u8 {
        self.wiring[c as usize]
    }

    /// Get a reference to the plugboard's wiring.
    pub fn wiring(&self) -> &[u8; 26] {
        &self.wiring
    }

    /// Return value is true if unplugged.
    pub fn unplugged(&self) -> [bool; 26] {
        let mut ret_val = [true; 26];

        for (idx, other) in self.wiring.iter().enumerate() {
            ret_val[idx] = idx == *other as usize;
        }

        ret_val
    }

    /// Essentially does the reverse of constructing a plugboard.
    /// This was needed because of how our implementations differ when storing
    /// the plugboard in the EnigmaKey. I stored them pre-decoded to make it cheaper
    /// to create and copy an EnigmaKey, whereas Mike Pound stored his encoded as a String,
    /// making it easier for his implementation to get the current plugboard.
    /// This function is only called like 10 times, so the cost isn't too bad.
    pub fn generate_connections(&self) -> Vec<(char, char)> {
        let mut connections = Vec::new();

        let mut seen = [false; 26];
        for (idx, other) in self.wiring.iter().enumerate() {
            if idx as u8 == *other || seen[idx] {
                continue; // Not connected or already seen.
            }

            seen[idx] = true;
            seen[*other as usize] = true;

            let a = (idx as u8 + b'A') as char;
            let b = (*other + b'A') as char;
            connections.push((a, b));
        }

        connections
    }
}

impl Display for Plugboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut seen = [false; 26];

        let mut wiring_iter = self
            .wiring
            .iter()
            .enumerate()
            .filter(|(i, other)| *i != **other as usize);

        if let Some((idx, other)) = wiring_iter.next() {
            seen[idx] = true;
            seen[*other as usize] = true;

            let a = (idx as u8 + b'A') as char;
            let b = (*other + b'A') as char;

            f.write_char(a)?;
            f.write_char(b)?;
        }

        for (idx, other) in wiring_iter {
            if seen[idx] {
                continue; // We can ignore the other half as they always come in pairs.
            }

            seen[idx] = true;
            seen[*other as usize] = true;

            let a = (idx as u8 + b'A') as char;
            let b = (*other + b'A') as char;

            f.write_char(' ')?;
            f.write_char(a)?;
            f.write_char(b)?;
        }

        Ok(())
    }
}

pub struct Enigma {
    left_rotor: Rotor,
    middle_rotor: Rotor,
    right_rotor: Rotor,
    reflector: ReflectorId,
    plugboard: Plugboard,
}

impl Enigma {
    pub fn new(key: EnigmaKey, reflector: ReflectorId) -> Self {
        Self {
            left_rotor: key.left_rotor,
            middle_rotor: key.middle_rotor,
            right_rotor: key.right_rotor,
            reflector,
            plugboard: key.plugboard,
        }
    }

    fn rotate(&mut self) {
        // If middle rotor notch - double-stepping
        if self.middle_rotor.is_at_notch() {
            self.middle_rotor.turnover();
            self.left_rotor.turnover();
        }
        // If left-rotor notch
        else if self.right_rotor.is_at_notch() {
            self.middle_rotor.turnover();
        }

        self.right_rotor.turnover();
    }

    pub fn encrypt(&mut self, c: char) -> char {
        assert!(c.is_ascii_uppercase());
        let mut c = c as u8 - b'A';

        self.rotate();

        // Plugboard in
        c = self.plugboard.forward(c);

        // Right to left
        c = self.right_rotor.forward(c);
        c = self.middle_rotor.forward(c);
        c = self.left_rotor.forward(c);

        // Reflector
        c = self.reflector.forward(c);

        // Left to right
        c = self.left_rotor.backward(c);
        c = self.middle_rotor.backward(c);
        c = self.right_rotor.backward(c);

        // Plugboard out
        c = self.plugboard.forward(c);

        (c + b'A') as char
    }
}
