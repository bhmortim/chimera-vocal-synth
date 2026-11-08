//! Real-time factor of the voices (single core): `cargo run --release -p chimera --example bench`.
use chimera::anatomy::human::BODY_MALE;
use chimera::control::score::Gesture;
use chimera::voice::detail::{Detail, Resolution};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};
use std::time::Instant;

const FS: f64 = 48000.0;

/// Speak the golden speech scenario for 4 s: (% of one core in real time, ms per 128-sample block).
fn bench(mut voice: HumanVoice, gs: &[Gesture]) -> (f64, f64) {
    voice.set_source(&SourcePatch::f0(110.0));
    let mut g2 = gs.to_vec();
    for g in g2.iter_mut() {
        g.t0 += 0.03;
        g.t1 += 0.03;
    }
    voice.schedule(&g2);
    voice.set_gate(1.0);
    let secs = 4.0;
    let n = (FS * secs) as usize;
    let mut out = vec![0.0f32; 128];
    let t0 = Instant::now();
    for _ in 0..n / 128 {
        voice.render(&mut out);
    }
    let dt = t0.elapsed().as_secs_f64();
    (100.0 * dt / secs, 1000.0 * dt / (n / 128) as f64)
}

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"));
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{root}/tests/golden/voice.json")).unwrap()).unwrap();
    let s = v.as_array().unwrap().iter().find(|s| s["name"] == "speech-male").unwrap();
    let gs: Vec<Gesture> = serde_json::from_value(s["ops"][0]["gestures"].clone()).unwrap();
    for os in [2usize, 1] {
        let (pct, ms) = bench(HumanVoice::new(FS, BODY_MALE, os), &gs);
        println!("human male os={os}: {pct:.1}% of one core in real time ({ms:.2} ms per 128-sample block)");
    }
    // the finer details (voice/detail.rs); os=4 on the standard grid separates the grid's share
    let (pct, ms) = bench(HumanVoice::new(FS, BODY_MALE, 4), &gs);
    println!("human male os=4, standard grid: {pct:.1}% ({ms:.2} ms per block)");
    for resolution in [Resolution::High, Resolution::Ultra] {
        for anatomy in [false, true] {
            let d = Detail { resolution, anatomy };
            let (pct, ms) = bench(HumanVoice::with_detail(FS, BODY_MALE, d), &gs);
            println!("human male {resolution:?}{} (os={}): {pct:.1}% ({ms:.2} ms per block)", if anatomy { " + anatomy" } else { "" }, d.oversample(FS));
        }
    }
}
