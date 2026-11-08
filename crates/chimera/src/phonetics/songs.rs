//! Demo melodies for Sing mode: [MIDI note, beats] per syllable, in the adult female register.

use serde::Serialize;

/// octave shift per body
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongOctave {
    pub male: f64,
    pub female: f64,
    pub child: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoSong {
    pub name: &'static str,
    pub bpm: f64,
    pub lyrics: &'static str,
    pub notes: &'static [[f64; 2]],
    /// octave shift per body
    pub octave: SongOctave,
}

#[rustfmt::skip]
pub static DEMO_SONGS: &[DemoSong] = &[
    DemoSong {
        // an original tune for this instrument
        name: "Made of air",
        bpm: 84.0,
        lyrics: "Breathe in, breathe out, I am made of air. Tongue and lips and throat, singing anywhere.",
        notes: &[
            [64.0, 1.0], [60.0, 1.0], [62.0, 1.0], [57.0, 1.0],
            [57.0, 0.5], [60.0, 0.5], [64.0, 1.0], [62.0, 0.5], [60.0, 1.5],
            [65.0, 1.0], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [62.0, 1.5],
            [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [59.0, 0.5], [57.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // traditional (public domain)
        name: "Twinkle, twinkle",
        bpm: 100.0,
        lyrics: "Twinkle twinkle little star, how I wonder what you are.",
        notes: &[
            [60.0, 1.0], [60.0, 1.0], [67.0, 1.0], [67.0, 1.0], [69.0, 1.0], [69.0, 1.0], [67.0, 2.0],
            [65.0, 1.0], [65.0, 1.0], [64.0, 1.0], [64.0, 1.0], [62.0, 1.0], [62.0, 1.0], [60.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    // The four below are public domain, one verse each. A lone "-" holds the vowel over the next
    // note (a melisma); a line break ends a phrase (a breath), as punctuation does, and can follow a
    // held note, which punctuation can't.
    DemoSong {
        // words John Newton (1779); the tune "New Britain" as in E. O. Excell's hymnal (1900), in F
        name: "Amazing Grace",
        bpm: 69.0,
        lyrics: "Amazing - grace how sweet the sound\nthat saved a - wretch like me\nI once - was - lost but now - am - found\nwas blind but - now I see",
        notes: &[
            [60.0, 1.0], [65.0, 2.0], [69.0, 0.5], [65.0, 0.5], [69.0, 2.0], [67.0, 1.0], [65.0, 2.0], [62.0, 1.0], [60.0, 2.0],
            [60.0, 1.0], [65.0, 2.0], [69.0, 0.5], [65.0, 0.5], [69.0, 2.0], [67.0, 1.0], [72.0, 2.0],
            [69.0, 1.0], [72.0, 1.5], [69.0, 0.5], [72.0, 0.5], [69.0, 0.5], [65.0, 2.0], [60.0, 1.0], [62.0, 1.5], [65.0, 0.5], [65.0, 0.5], [62.0, 0.5], [60.0, 2.0],
            [60.0, 1.0], [65.0, 2.0], [69.0, 0.5], [65.0, 0.5], [69.0, 2.0], [67.0, 1.0], [65.0, 3.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // words Frederic Weatherly (1913) to the Londonderry Air (traditional, printed 1855), in C;
        // the rests before each pickup are folded into the note before them
        name: "Danny Boy",
        bpm: 66.0,
        lyrics: "Oh Danny boy the pipes the pipes are calling -\nfrom glen to glen and down the mountain side\nthe summer's gone and all the roses falling -\n'tis you 'tis you must go and I must bide - -\nbut come ye back when summer's in the meadow -\nor when the valley's hushed and white with snow\n'tis I'll be here in sunshine or in shadow -\noh Danny boy oh Danny boy I love you so",
        notes: &[
            [59.0, 0.5], [60.0, 0.5], [62.0, 0.5], [64.0, 1.5], [62.0, 0.5], [64.0, 0.5], [69.0, 0.5], [67.0, 0.5], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [57.0, 1.5],
            [60.0, 0.5], [64.0, 0.5], [65.0, 0.5], [67.0, 1.5], [69.0, 0.5], [67.0, 0.5], [64.0, 0.5], [60.0, 0.5], [64.0, 0.5], [62.0, 2.5],
            [59.0, 0.5], [60.0, 0.5], [62.0, 0.5], [64.0, 1.5], [62.0, 0.5], [64.0, 0.5], [69.0, 0.5], [67.0, 0.5], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [57.0, 0.5],
            [56.0, 0.5], [57.0, 0.5], [59.0, 0.5], [60.0, 0.5], [62.0, 0.5], [64.0, 1.5], [65.0, 0.5], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [62.0, 0.5], [60.0, 2.5],
            [67.0, 0.5], [69.0, 0.5], [71.0, 0.5], [72.0, 1.5], [71.0, 0.5], [71.0, 0.5], [69.0, 0.5], [67.0, 0.5], [64.0, 0.5], [67.0, 0.5], [64.0, 0.5], [60.0, 1.5],
            [67.0, 0.5], [69.0, 0.5], [71.0, 0.5], [72.0, 1.5], [71.0, 0.5], [71.0, 0.5], [69.0, 0.5], [67.0, 0.5], [64.0, 0.5], [62.0, 2.5],
            [67.0, 0.5], [67.0, 0.5], [67.0, 0.5], [76.0, 1.5], [74.0, 0.5], [74.0, 0.5], [72.0, 0.5], [69.0, 0.5], [72.0, 0.5], [67.0, 0.5], [64.0, 0.5], [60.0, 1.5],
            [59.0, 0.5], [60.0, 0.5], [62.0, 0.5], [64.0, 0.5], [69.0, 0.5], [67.0, 0.5], [64.0, 0.5], [62.0, 0.5], [60.0, 0.5], [57.0, 0.5], [59.0, 0.5], [60.0, 2.5],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // traditional English (16th century), verse and refrain, in E minor; beats are quarter notes
        // of its 6/8
        name: "Greensleeves",
        bpm: 76.0,
        lyrics: "Alas my love - you do me wrong - to cast me off - discourteously\nfor I have loved - you well and long - delighting - in - your company\nGreen-sleeves - was all my joy - - Green - sleeves - was my delight\nGreen-sleeves was my heart of gold - and who but my lady - Green-sleeves",
        notes: &[
            [64.0, 0.5], [67.0, 1.0], [69.0, 0.5], [71.0, 0.75], [73.0, 0.25], [71.0, 0.5], [69.0, 1.0], [66.0, 0.5], [62.0, 0.75], [64.0, 0.25],
            [66.0, 0.5], [67.0, 1.0], [64.0, 0.5], [64.0, 0.75], [63.0, 0.25], [64.0, 0.5], [66.0, 1.0], [63.0, 0.25], [63.0, 0.25], [59.0, 1.0],
            [64.0, 0.5], [67.0, 1.0], [69.0, 0.5], [71.0, 0.75], [73.0, 0.25], [71.0, 0.5], [69.0, 1.0], [66.0, 0.5], [62.0, 0.75], [64.0, 0.25],
            [66.0, 0.5], [67.0, 0.75], [66.0, 0.25], [64.0, 0.5], [63.0, 0.75], [61.0, 0.25], [63.0, 0.5], [64.0, 1.0], [64.0, 0.5], [64.0, 1.5],
            [74.0, 1.5], [74.0, 0.75], [73.0, 0.25], [71.0, 0.5], [69.0, 1.0], [66.0, 0.5], [62.0, 0.75], [64.0, 0.25], [66.0, 0.5],
            [67.0, 1.0], [64.0, 0.5], [64.0, 0.75], [63.0, 0.25], [64.0, 0.5], [66.0, 1.0], [63.0, 0.5], [59.0, 1.5],
            [74.0, 1.5], [74.0, 0.75], [73.0, 0.25], [71.0, 0.5], [69.0, 1.0], [66.0, 0.5], [62.0, 0.75], [64.0, 0.25], [66.0, 0.5],
            [67.0, 0.75], [66.0, 0.25], [64.0, 0.5], [63.0, 0.75], [61.0, 0.25], [63.0, 0.5], [64.0, 1.5], [64.0, 2.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
    DemoSong {
        // traditional English ballad: the first verse and the tune as usually sung, in D Dorian
        name: "Scarborough Fair",
        bpm: 100.0,
        lyrics: "Are you going to Scarborough Fair?\nParsley sage rosemary and thyme\nremember me to one who lives there\nshe once was a true love of mine",
        notes: &[
            [62.0, 2.0], [62.0, 1.0], [69.0, 0.5], [69.0, 1.5], [69.0, 1.0], [64.0, 1.5], [65.0, 0.5], [64.0, 1.0], [62.0, 4.0],
            [69.0, 1.0], [72.0, 1.0], [74.0, 2.0], [72.0, 1.0], [69.0, 1.0], [71.0, 1.0], [67.0, 1.0], [69.0, 2.0],
            [74.0, 1.0], [74.0, 2.0], [74.0, 1.0], [72.0, 2.0], [69.0, 1.0], [69.0, 1.0], [67.0, 1.0], [65.0, 1.0], [64.0, 3.0],
            [62.0, 2.0], [69.0, 1.0], [67.0, 2.0], [65.0, 1.0], [64.0, 1.0], [62.0, 1.0], [60.0, 1.0], [62.0, 3.0],
        ],
        octave: SongOctave { male: -1.0, female: 0.0, child: 0.0 },
    },
];
