//! The finer voice models (`Detail`): the standard detail is the default voice, sample for
//! sample; finer resolutions shorten the sections within the stability limit; with each
//! detail's own calibrated vowels they keep the standard model's formants; the extra anatomy
//! shapes the spectrum above the formants; every detail plays steadily; and the engine switches
//! detail keeping the performer's settings.
use chimera::airway::frequency::{find_peaks, transfer_function};
use chimera::anatomy::human::{BODY_CHILD, BODY_MALE, BodyMix, GRID_CAPACITY, GridLines, HumanArtic, HumanBody, NEUTRAL};
use chimera::engine::Engine;
use chimera::phonetics::planner::{vowel_artic, vowel_artic_at, vowel_table_for};
use chimera::voice::detail::{Detail, Resolution};
use chimera::voice::human_airway::HumanAirway;
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const FS_OUT: f64 = 48000.0;

fn every_detail() -> Vec<Detail> {
    Resolution::ALL.iter().flat_map(|&resolution| [false, true].map(|anatomy| Detail { resolution, anatomy })).collect()
}

/// The transfer function (dB) and the first formants of a vowel, as the UI computes them, the
/// vowel planned as `targets` calibrated it.
fn spectrum_with(detail: Detail, targets: Detail, body: BodyMix, vowel: &str) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let a: HumanArtic = vowel_artic_at(vowel, body, targets).unwrap().apply_to(&NEUTRAL);
    let fs = FS_OUT * detail.oversample(FS_OUT) as f64;
    let mut w = HumanAirway::with_detail(fs, body.body(), detail);
    w.apply(&a, 0.0);
    let el = w.net.linear_elements();
    let freqs: Vec<f64> = (0..360).map(|i| 60.0 * (8000.0f64 / 60.0).powf(i as f64 / 359.0)).collect();
    let h = transfer_function(&el, w.pharynx.first_node(), &freqs, &[w.glottis_port]);
    let db = h.mag.iter().map(|x| 20.0 * (x + 1e-12).log10()).collect();
    let formants = find_peaks(&freqs, &h.mag, 150.0).iter().map(|p| p.freq).take(3).collect();
    (freqs, db, formants)
}

/// [`spectrum_with`] the detail's own calibrated vowels, as the app plans them.
fn spectrum(detail: Detail, body: BodyMix, vowel: &str) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    spectrum_with(detail, detail, body, vowel)
}

#[test]
fn the_standard_detail_is_the_default_voice_sample_for_sample() {
    assert_eq!(GridLines::scaled(1.0), GridLines::STANDARD);
    let mut a = HumanVoice::new(FS_OUT, BODY_MALE, 2);
    let mut b = HumanVoice::with_detail(FS_OUT, BODY_MALE, Detail::STANDARD);
    a.set_gate(1.0);
    b.set_gate(1.0);
    let (mut xa, mut xb) = (vec![0.0f32; 4800], vec![0.0f32; 4800]);
    for block in 0..5 {
        a.render(&mut xa);
        b.render(&mut xb);
        assert_eq!(xa, xb, "block {block}");
    }
}

#[test]
fn finer_resolutions_shorten_the_sections_within_the_stability_limit() {
    assert_eq!(Resolution::ALL.map(|resolution| Detail { resolution, anatomy: false }.oversample(48000.0)), [2, 3, 4]);
    assert_eq!(Resolution::ALL.map(|resolution| Detail { resolution, anatomy: false }.oversample(96000.0)), [1, 2, 2]);
    for body in [BODY_MALE, BODY_CHILD] {
        let mut prev = (0, 0);
        for resolution in Resolution::ALL {
            for anatomy in [false, true] {
                let d = Detail { resolution, anatomy };
                let w = HumanAirway::with_detail(FS_OUT * d.oversample(FS_OUT) as f64, body, d);
                let cfl = w.net.check_cfl();
                assert!(cfl <= 0.95, "{d:?}: Courant number {cfl:.3}");
                assert!(w.tract.lines.samples() <= GRID_CAPACITY, "{d:?}: the grid outgrows the frame's reserve");
                if !anatomy {
                    assert!(w.n_ph > prev.0 && w.n_or > prev.1, "{d:?}: {} + {} sections after {prev:?}", w.n_ph, w.n_or);
                    prev = (w.n_ph, w.n_or);
                }
            }
        }
    }
}

#[test]
fn the_finer_models_keep_the_calibrated_formants() {
    let mut worst = [(0.0f64, String::new()), (0.0f64, String::new())];
    for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
        for v in ["i", "a", "u"] {
            let (_, _, standard) = spectrum(Detail::STANDARD, body, v);
            for d in every_detail().into_iter().skip(1) {
                let (_, _, f) = spectrum(d, body, v);
                let shifts: Vec<f64> = (0..2).map(|i| 100.0 * (f[i] - standard[i]) / standard[i]).collect();
                eprintln!("{:<7} /{v}/ {:<6?} {:<5} F1 {:+5.1}%  F2 {:+5.1}%", format!("{body:?}").split(',').next().unwrap_or(""), d.resolution, if d.anatomy { "+anat" } else { "" }, shifts[0], shifts[1]);
                for i in 0..2 {
                    if shifts[i].abs() > worst[i].0 {
                        worst[i] = (shifts[i].abs(), format!("{body:?} /{v}/ {d:?}: F{} {:.0} Hz vs {:.0} Hz", i + 1, f[i], standard[i]));
                    }
                }
            }
        }
    }
    eprintln!("worst F1: {:.1}% ({})\nworst F2: {:.1}% ({})", worst[0].0, worst[0].1, worst[1].0, worst[1].1);
    // Each finer detail plans its vowels from its own table, refit on its airway to the formants
    // the standard model gives them (`chimera calibrate vowels --detail <name>`). With the
    // standard table instead, F2 fell up to 6.6% (back rounded vowels at ultra: the finer grid
    // finds the narrowest point of constrictions the standard grid straddles); see below.
    assert!(worst[0].0 < 2.0, "F1 moved {:.1}%: {}", worst[0].0, worst[0].1);
    assert!(worst[1].0 < 2.0, "F2 moved {:.1}%: {}", worst[1].0, worst[1].1);
}

#[test]
fn plans_take_their_vowels_from_the_voices_detail() {
    use chimera::anatomy::human::ArticKey;
    use chimera::phonetics::ipa::tokenize_ipa;
    use chimera::phonetics::planner::{PlanOptions, plan_utterance};
    let segs = tokenize_ipa("ˈu");
    // the planned /u/'s tongue-body height, and the table's for that detail
    let planned = |detail: Detail| -> f64 {
        let plan = plan_utterance(&segs, &PlanOptions { body: BodyMix::CHILD, t0: 0.0, detail, ..Default::default() });
        plan.gestures.iter().filter(|g| g.label.as_ref().is_some_and(|l| l.as_str() == "u")).find_map(|g| g.targets.get(ArticKey::TbY)).expect("a /u/ gesture with a tongue-body target")
    };
    let table = |detail: Detail| vowel_artic_at("u", BodyMix::CHILD, detail).unwrap().get(ArticKey::TbY).unwrap();
    let ultra = Detail { resolution: Resolution::Ultra, anatomy: true };
    assert_eq!(planned(Detail::STANDARD), table(Detail::STANDARD));
    assert_eq!(planned(ultra), table(ultra));
    assert_ne!(table(ultra), table(Detail::STANDARD), "the refit moved /u/ for this detail");
}

#[test]
fn every_detail_has_its_own_calibrated_vowels() {
    let standard = vowel_table_for(Detail::STANDARD);
    for d in every_detail().into_iter().skip(1) {
        let t = vowel_table_for(d);
        // the same bodies and vowels as the standard table
        assert_eq!(t.keys().collect::<Vec<_>>(), standard.keys().collect::<Vec<_>>(), "{d:?}");
        for (body, vowels) in t {
            assert_eq!(vowels.keys().collect::<Vec<_>>(), standard[body].keys().collect::<Vec<_>>(), "{d:?} {body}");
        }
        // and refit, not a copy of the standard's postures
        let refit = t.iter().flat_map(|(b, vs)| vs.iter().map(move |(v, x)| (b, v, x))).filter(|(b, v, x)| x.artic != standard[b.as_str()][v.as_str()].artic).count();
        assert!(refit > 0, "{d:?}: the table is the standard's");
    }
    // with the standard table, the finer models' formants drift: what the refit is for
    let (_, _, s) = spectrum(Detail::STANDARD, BodyMix::CHILD, "u");
    let (_, _, u) = spectrum_with(Detail { resolution: Resolution::Ultra, anatomy: false }, Detail::STANDARD, BodyMix::CHILD, "u");
    assert!((u[1] - s[1]).abs() / s[1] > 0.03, "child /u/ F2 at ultra on the standard table: {:.0} vs {:.0} Hz", u[1], s[1]);
}

#[test]
fn the_extra_anatomy_shapes_the_spectrum_above_the_formants() {
    for v in ["a", "i"] {
        // the same posture in both, so only the cavities differ
        let (freqs, standard, _) = spectrum(Detail::STANDARD, BodyMix::MALE, v);
        let (_, full, _) = spectrum_with(Detail { resolution: Resolution::Standard, anatomy: true }, Detail::STANDARD, BodyMix::MALE, v);
        let moved = freqs.iter().zip(standard.iter().zip(&full)).filter(|(f, _)| **f >= 3500.0).map(|(_, (a, b))| (a - b).abs()).fold(0.0f64, f64::max);
        assert!(moved > 3.0, "/{v}/: the side cavities moved the spectrum above 3.5 kHz by only {moved:.1} dB");
    }
}

#[test]
fn every_detail_plays_a_steady_vowel() {
    let bodies: [(HumanBody, BodyMix); 2] = [(BODY_MALE, BodyMix::MALE), (BODY_CHILD, BodyMix::CHILD)];
    for (body, mix) in bodies {
        for d in every_detail() {
            let mut v = HumanVoice::with_detail(FS_OUT, body, d);
            v.set_articulation(&vowel_artic("a", mix).unwrap(), None);
            v.set_gate(1.0);
            let mut out = vec![0.0f32; 4800];
            let (mut peak, mut energy) = (0.0f32, 0.0f64);
            for _ in 0..10 {
                v.render(&mut out);
                for &x in &out {
                    assert!(x.is_finite(), "{mix:?} {d:?}: a non-finite sample");
                    peak = peak.max(x.abs());
                    energy += (x as f64) * (x as f64);
                }
            }
            assert!(peak < 1.0 && energy > 1e-3, "{mix:?} {d:?}: peak {peak}, energy {energy}");
        }
    }
}

#[test]
fn the_engine_switches_detail_keeping_the_performers_settings() {
    let mut e = Engine::new(FS_OUT);
    e.set_body(BodyMix::FEMALE);
    e.set_source(&SourcePatch::f0(150.0));
    let before = e.info();
    let ultra = Detail { resolution: Resolution::Ultra, anatomy: true };
    let info = e.set_detail(ultra);
    assert_eq!(e.detail(), ultra);
    assert_eq!(info.body, before.body);
    assert!(info.n_ph > before.n_ph && info.n_or > before.n_or, "{before:?} → {info:?}");
    assert_eq!(e.voice().m, 4);
    assert_eq!(e.voice().source.f0, 150.0, "the pitch carries over");
    // a body switch keeps the detail
    e.set_body(BodyMix::MALE);
    assert_eq!((e.detail(), e.voice().m), (ultra, 4));
    // and the standard detail is the default voice again
    e.set_detail(Detail::STANDARD);
    assert_eq!(e.voice().m, 2);
    let mut out = vec![0.0f32; 1024];
    e.set_gate(1.0);
    e.render(&mut out);
    assert!(out.iter().all(|x| x.is_finite()));
}
