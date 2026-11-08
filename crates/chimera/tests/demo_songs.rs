//! The demo songs sing as written: one syllable per note (the UI pairs note i with syllable i and
//! would wrap round a short lyric; a lone "-" is a held syllable, so it counts), and each song's
//! plan fits what the worklet can schedule, for every body.
use chimera::anatomy::human::BodyMix;
use chimera::engine::MAX_SCHEDULE;
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song};
use chimera::phonetics::songs::DEMO_SONGS;
use chimera::phonetics::syllables::lyrics_to_syllables;
use std::io::Read;

/// The pronouncing dictionary, as the app loads it before a demo.
fn g2p() -> EnglishG2P {
    let path = format!("{}/../../public/data/cmudict.txt.gz", env!("CARGO_MANIFEST_DIR"));
    let gz = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut text = String::new();
    flate2::read::GzDecoder::new(&gz[..]).read_to_string(&mut text).unwrap();
    let mut g = EnglishG2P::new();
    g.load_dict(&text);
    g
}

#[test]
fn every_demo_song_has_a_syllable_per_note() {
    let g = g2p();
    let mut bad = Vec::new();
    for song in DEMO_SONGS {
        let syl = lyrics_to_syllables(song.lyrics, &g);
        if syl.len() != song.notes.len() {
            let text: Vec<&str> = syl.iter().map(|s| s.text.as_str()).collect();
            bad.push(format!("{}: {} syllables for {} notes: {}", song.name, syl.len(), song.notes.len(), text.join(" ")));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn every_demo_song_plans_within_the_schedule() {
    let g = g2p();
    for song in DEMO_SONGS {
        let syl = lyrics_to_syllables(song.lyrics, &g);
        for (body, octave) in [(BodyMix::MALE, song.octave.male), (BodyMix::FEMALE, song.octave.female), (BodyMix::CHILD, song.octave.child)] {
            let notes: Vec<SungNote> = song.notes.iter().zip(&syl).map(|(n, s)| SungNote { midi: n[0] + 12.0 * octave, beats: n[1], syl: s.clone() }).collect();
            let plan = plan_song(&notes, &SongOptions { body, bpm: song.bpm, t0: 0.35, vibrato: 30.0 });
            assert!(plan.gestures.len() <= MAX_SCHEDULE, "{} ({body:?}): {} gestures; the worklet schedules {MAX_SCHEDULE}", song.name, plan.gestures.len());
            assert!(plan.end.is_finite() && plan.end > 0.0, "{} ({body:?}): ends at {}", song.name, plan.end);
            if body == BodyMix::FEMALE {
                eprintln!("{}: {} notes, {} gestures, {:.1} s", song.name, notes.len(), plan.gestures.len(), plan.end);
            }
        }
    }
}
