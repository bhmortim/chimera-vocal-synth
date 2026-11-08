//! `chimera calibrate …`: ports of `tools/calibrate-vowels.ts` and `tools/calibrate-consonants.ts`.
//! They regenerate the tables the engine embeds (`crates/chimera/src/phonetics/*.generated.json`),
//! written as `JSON.stringify(table, null, 1)` wrote them.

use crate::js::{Json, fixed, pad_end, pad_start};
use crate::repo_path;
use chimera::airway::frequency::{find_peaks, transfer_function};
use chimera::anatomy::human::{
    ArticKey, BODY_CHILD, BODY_FEMALE, BODY_MALE, HumanArtic, HumanBody, NEUTRAL, rest_tip,
};
use chimera::math::{js_max, js_min, js_round, ln, pow};
use chimera::phonetics::inventory::consonants as inventory;
use chimera::phonetics::planner::VowelTable;
use chimera::phonetics::vowel_data::VOWELS;
use chimera::util::optimize::{NelderMeadOpts, nelder_mead};
use chimera::voice::detail::Detail;
use chimera::voice::human_airway::HumanAirway;

const VOWEL_TABLE: &str = "crates/chimera/src/phonetics/vowelTargets.generated.json";

/// A detail's vowel table: the standard model's, or `vowelTargets.<detail>.generated.json`.
fn vowel_table_path(d: Detail) -> String {
    if d == Detail::STANDARD {
        VOWEL_TABLE.to_string()
    } else {
        format!("crates/chimera/src/phonetics/vowelTargets.{}.generated.json", d.name())
    }
}

/// Split `--detail <name>` from the arguments (the standard model when absent).
fn detail_option(a: &[String]) -> Result<(Vec<String>, Detail), String> {
    let mut rest = Vec::new();
    let mut detail = Detail::STANDARD;
    let mut it = a.iter();
    while let Some(x) = it.next() {
        if x == "--detail" {
            let name = it.next().ok_or("--detail needs a name")?;
            detail = Detail::from_name(name).ok_or_else(|| {
                let names: Vec<&str> = Detail::ALL.iter().map(|d| d.name()).collect();
                format!("unknown detail {name} (one of {})", names.join(", "))
            })?;
        } else {
            rest.push(x.clone());
        }
    }
    Ok((rest, detail))
}
const CONSONANT_TABLE: &str = "crates/chimera/src/phonetics/consonantTargets.generated.json";
const BODIES: [(&str, HumanBody); 3] = [
    ("male", BODY_MALE),
    ("female", BODY_FEMALE),
    ("child", BODY_CHILD),
];

/// Split `--out <file>` from the positional arguments.
fn out_option(a: &[String], default: &str) -> Result<(Vec<String>, String), String> {
    let mut rest = Vec::new();
    let mut out = repo_path(default);
    let mut it = a.iter();
    while let Some(x) = it.next() {
        if x == "--out" {
            out = it.next().ok_or("--out needs a file")?.clone();
        } else {
            rest.push(x.clone());
        }
    }
    Ok((rest, out))
}

fn round3(x: f64) -> f64 {
    js_round(x * 1000.0) / 1000.0
}

/// tbX tbY jaw lipAp lipProt lipWidth root larynx curl
const LO: [f64; 9] = [1.2, 5.0, 0.05, 0.12, 0.0, 0.0, -0.9, -1.3, 0.0];
const HI: [f64; 9] = [4.6, 8.8, 2.3, 2.0, 1.4, 1.0, 0.9, 1.3, 1.0];

fn to_artic(p: &[f64]) -> HumanArtic {
    let (tb_x, tb_y, jaw, lip_ap, lip_prot, lip_width, root, larynx, curl) =
        (p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8]);
    let tip = rest_tip(jaw, tb_x, tb_y);
    // rhotic curl raises the tip toward the post-alveolar region
    let tt_x = tip.x - 0.4 * curl;
    let tt_y = tip.y + 2.0 * curl;
    HumanArtic {
        tb_x,
        tb_y,
        jaw,
        lip_ap,
        lip_prot,
        lip_width,
        root,
        larynx,
        tt_x,
        tt_y,
        tt_curl: curl,
        ..NEUTRAL
    }
}

/// Resonance peaks of the airway in posture `a` (frequency-domain model), and its narrowest area.
fn formants(air: &mut HumanAirway, freqs: &[f64], a: &HumanArtic) -> (Vec<f64>, f64) {
    air.apply(a, 0.0);
    let el = air.net.linear_elements();
    let h = transfer_function(&el, air.pharynx.first_node(), freqs, &[air.glottis_port]);
    let pk: Vec<f64> = find_peaks(freqs, &h.mag, 150.0)
        .iter()
        .map(|p| p.freq)
        .collect();
    let mut min_area = f64::INFINITY;
    for &x in air.ph_areas.iter().chain(air.or_areas.iter()) {
        min_area = js_min(min_area, x);
    }
    (pk, min_area)
}

/// Articulatory vowel targets fitted to reference formants with the frequency-domain airway
/// model. `calibrate vowels [male|female|child|all] [--detail <name>] [--out file]`
///
/// A finer detail (`--detail high`, `ultra-anatomy`, …; see `voice/detail.rs`) is refit on its
/// own airway to the formants the standard model gives each vowel, starting from the standard
/// posture, so switching detail leaves each vowel where it was. Its table goes beside the
/// standard one, `vowelTargets.<detail>.generated.json`, and needs the standard table first.
pub fn vowels(a: &[String]) -> Result<(), String> {
    let (a, detail) = detail_option(a)?;
    let (rest, out) = out_option(&a, &vowel_table_path(detail))?;
    let which = rest.first().map(String::as_str).unwrap_or("all");
    let standard: Option<VowelTable> = if detail == Detail::STANDARD {
        None
    } else {
        let p = repo_path(VOWEL_TABLE);
        let text = std::fs::read_to_string(&p).map_err(|e| format!("{p}: {e}"))?;
        Some(serde_json::from_str(&text).map_err(|e| format!("{p}: {e}"))?)
    };
    let n = 700;
    let freqs: Vec<f64> = (0..n)
        .map(|i| 120.0 * pow(4600.0 / 120.0, i as f64 / (n - 1) as f64))
        .collect();
    let mut results = match std::fs::read_to_string(&out) {
        Ok(text) => Json::parse(&text).map_err(|e| format!("{out}: {e}"))?,
        Err(_) => Json::Obj(Vec::new()),
    };

    for (bname, body) in BODIES {
        if which != "all" && which != bname {
            continue;
        }
        // the standard model is 2× at 48 kHz: 96 kHz, as this tool always fitted it
        let mut air = HumanAirway::with_detail(48000.0 * detail.oversample(48000.0) as f64, body, detail);
        let k = body.scale;
        let mut table: Vec<(String, Json)> = Vec::new();
        for vw in VOWELS {
            let reference = match bname {
                "male" => vw.male,
                "female" => vw.female,
                _ => vw.child,
            };
            // a finer detail fits the standard model's formants for this vowel, from its posture
            let fit = standard.as_ref().and_then(|s| s.get(bname)).and_then(|t| t.get(vw.ipa)).filter(|v| v.formants.len() >= 3);
            let t: [f64; 3] = match fit {
                Some(v) => [v.formants[0], v.formants[1], v.formants[2]],
                None => reference,
            };
            // initial guess from vowel-space position
            let mut x0 = vec![
                3.5 - 1.3 * vw.back + 0.2 * vw.height * (1.0 - vw.back),
                8.0 - 2.0 * vw.height - 0.25 * vw.back,
                0.25 + 1.4 * vw.height,
                if vw.round {
                    0.35 + 0.6 * vw.height
                } else {
                    0.6 + 1.0 * vw.height
                },
                if vw.round { 0.9 } else { 0.1 },
                if vw.round { 0.12 } else { 0.85 },
                0.4 * (1.0 - vw.back) - 0.5 * vw.height * vw.back,
                0.4 * (1.0 - vw.height) * (1.0 - vw.back) - if vw.round { 0.4 } else { 0.0 },
                if vw.ipa == "ɝ" { 0.6 } else { 0.0 },
            ];
            if vw.ipa == "ɯ" {
                x0 = vec![2.6, 7.35, 0.25, 0.5, 0.1, 0.65, 0.4, 0.2, 0.0];
            }
            let jaw_prior = 0.2 + 1.5 * vw.height;
            const KEYS: [ArticKey; 9] = [
                ArticKey::TbX,
                ArticKey::TbY,
                ArticKey::Jaw,
                ArticKey::LipAp,
                ArticKey::LipProt,
                ArticKey::LipWidth,
                ArticKey::Root,
                ArticKey::Larynx,
                ArticKey::TtCurl,
            ];
            // a refit starts from, and stays near, the standard posture
            let anchor: Option<Vec<f64>> = fit.map(|v| KEYS.iter().zip(&x0).map(|(key, d)| v.artic.get(*key).unwrap_or(*d)).collect());
            let start = anchor.clone().unwrap_or_else(|| x0.clone());
            let mut cost = |p: &[f64]| -> f64 {
                let a = to_artic(p);
                let (f, min_area) = formants(&mut air, &freqs, &a);
                if f.len() < 3 {
                    return 50.0;
                }
                let mut c = 0.0;
                let w = [1.0, 1.0, if vw.ipa == "ɝ" { 1.2 } else { 0.45 }];
                for i in 0..3 {
                    c += w[i] * pow(ln(f[i] / t[i]), 2.0);
                }
                // physiology priors (soft)
                let min_ok = 0.22 * k * k;
                if min_area < min_ok {
                    c += 0.4 * pow((min_ok - min_area) / min_ok, 2.0);
                }
                // A refit's question is narrower: the nearest posture to the standard one (which
                // the priors below already balanced) that gives its formants on this airway. With
                // those priors instead, a vowel whose standard posture they had traded against a
                // reference it never reached (female /a/: lips at their limit, larynx raised)
                // slid back towards them and lost 4–5 % of F1.
                if let Some(p0) = &anchor {
                    for j in 0..9 {
                        c += 0.003 * pow(p[j] - p0[j], 2.0);
                    }
                    if vw.ipa != "ɝ" {
                        c += 0.5 * p[8] * p[8];
                    }
                    return c;
                }
                c += 0.004 * pow(p[2] - jaw_prior, 2.0);
                let open = vw.height > 0.6;
                if vw.round {
                    let (w_max, p_min, ap_max) = (
                        if open { 0.45 } else { 0.3 },
                        if open { 0.35 } else { 0.6 },
                        0.3 + 0.7 * vw.height,
                    );
                    c += 0.05 * pow(js_max(0.0, p[5] - w_max), 2.0)
                        + 0.05 * pow(js_max(0.0, p_min - p[4]), 2.0)
                        + 0.05 * pow(js_max(0.0, p[3] - ap_max), 2.0);
                } else {
                    let ap_prior = 0.35 + 0.75 * p[2];
                    c += 0.05 * pow(js_max(0.0, 0.55 - p[5]), 2.0)
                        + 0.05 * pow(js_max(0.0, p[4] - 0.35), 2.0)
                        + 0.03 * pow(js_max(0.0, (p[3] - ap_prior).abs() - 0.35), 2.0);
                }
                c += 0.002 * p[6] * p[6] + 0.006 * p[7] * p[7];
                // the tongue body near where the vowel's height and backness put it: many postures
                // give the same formants, and without this the fit wandered (a schwa with the
                // tongue as low as /ɑ/), making every transition through it needlessly long
                c += 0.006 * (pow(p[0] - x0[0], 2.0) + pow(p[1] - x0[1], 2.0));
                if vw.ipa != "ɝ" {
                    c += 0.5 * p[8] * p[8];
                }
                c
            };
            let opts = |step: [f64; 9], max_iter: usize| NelderMeadOpts {
                lo: Some(LO.to_vec()),
                hi: Some(HI.to_vec()),
                step: Some(step.to_vec()),
                max_iter: Some(max_iter),
                tol: None,
            };
            // a refit searches near its start; a fit from the vowel-space guess searches wide
            let first_step = if anchor.is_some() {
                [0.15, 0.15, 0.12, 0.1, 0.12, 0.1, 0.12, 0.12, 0.05]
            } else {
                [0.4, 0.4, 0.3, 0.25, 0.3, 0.25, 0.3, 0.3, 0.2]
            };
            let mut best = nelder_mead(&mut cost, &start, opts(first_step, 1800));
            // restart from the best point to escape shallow minima
            for _ in 0..2 {
                let nb = nelder_mead(
                    &mut cost,
                    &best.x,
                    opts([0.2, 0.2, 0.15, 0.12, 0.15, 0.12, 0.15, 0.15, 0.1], 1000),
                );
                if nb.fx < best.fx {
                    best = nb;
                }
            }
            let a = to_artic(&best.x);
            let (f, _) = formants(&mut air, &freqs, &a);
            let err: Vec<f64> = (0..3)
                .map(|i| match f.get(i) {
                    Some(&fi) if fi != 0.0 && !fi.is_nan() => (100.0 * (fi - t[i])) / t[i],
                    _ => f64::NAN,
                })
                .collect();
            let nums = |xs: &mut dyn Iterator<Item = f64>| Json::Arr(xs.map(Json::Num).collect());
            let artic = [
                ("tbX", a.tb_x),
                ("tbY", a.tb_y),
                ("jaw", a.jaw),
                ("lipAp", a.lip_ap),
                ("lipProt", a.lip_prot),
                ("lipWidth", a.lip_width),
                ("root", a.root),
                ("larynx", a.larynx),
                ("ttX", a.tt_x),
                ("ttY", a.tt_y),
                ("ttCurl", a.tt_curl),
            ];
            table.push((
                vw.ipa.to_string(),
                Json::Obj(vec![
                    (
                        "artic".into(),
                        Json::Obj(
                            artic
                                .iter()
                                .map(|(k, x)| (k.to_string(), Json::Num(round3(*x))))
                                .collect(),
                        ),
                    ),
                    (
                        "formants".into(),
                        nums(&mut f.iter().take(4).map(|&x| js_round(x))),
                    ),
                    ("target".into(), nums(&mut t.iter().map(|&x| js_round(x)))),
                    (
                        "err".into(),
                        nums(&mut err.iter().map(|&e| js_round(e * 10.0) / 10.0)),
                    ),
                ]),
            ));
            println!(
                "{} {} F={}  T={}  err%={}  tb=({},{}) jaw={} lip={}/{}/{} root={} lx={} curl={}",
                pad_end(bname, 6),
                pad_end(vw.ipa, 2),
                f.iter()
                    .take(3)
                    .map(|x| pad_start(&fixed(*x, 0), 5))
                    .collect::<String>(),
                t.iter()
                    .map(|x| pad_start(&fixed(*x, 0), 5))
                    .collect::<String>(),
                err.iter()
                    .map(|e| pad_start(&fixed(*e, 1), 6))
                    .collect::<String>(),
                fixed(a.tb_x, 2),
                fixed(a.tb_y, 2),
                fixed(a.jaw, 2),
                fixed(a.lip_ap, 2),
                fixed(a.lip_prot, 2),
                fixed(a.lip_width, 2),
                fixed(a.root, 2),
                fixed(a.larynx, 2),
                fixed(a.tt_curl, 2)
            );
        }
        results.set(bname, Json::Obj(table));
    }
    std::fs::write(&out, results.stringify(1)).map_err(|e| format!("{out}: {e}"))?;
    println!("wrote {out}");
    Ok(())
}

/// The main coordinate of each fricative set so the constriction area matches its target
/// (adult-male target × body scale²) in an /a/-like context: bisection on the monotonic contact
/// model. Reads the vowel table (its /ɑ/) from disk. `calibrate consonants [--out file]`
pub fn consonants(a: &[String]) -> Result<(), String> {
    let (_, out) = out_option(a, CONSONANT_TABLE)?;
    let vpath = repo_path(VOWEL_TABLE);
    let vtext = std::fs::read_to_string(&vpath).map_err(|e| format!("{vpath}: {e}"))?;
    let vtable: VowelTable = serde_json::from_str(&vtext).map_err(|e| format!("{vpath}: {e}"))?;
    let mut result: Vec<(String, Json)> = Vec::new();
    for (name, body) in BODIES {
        let mut air = HumanAirway::new(96000.0, body);
        let mut ctx = vtable[name]["ɑ"].artic.apply_to(&NEUTRAL);
        ctx.set(ArticKey::Jaw, 0.55);
        let mut table: Vec<(String, Json)> = Vec::new();
        for p in inventory() {
            let (Some(area), Some(key)) = (p.area, p.cal_key) else {
                continue;
            };
            if area == 0.0 || area.is_nan() {
                continue;
            }
            let target = area * body.scale * body.scale;
            let mut min_area = |v: f64| -> f64 {
                let mut a = p.targets.apply_to(&ctx);
                a.set(key, v);
                air.apply(&a, 0.0);
                let mut m = f64::INFINITY;
                for &x in air.ph_min.iter().chain(air.or_min.iter()) {
                    if x < m {
                        m = x;
                    }
                }
                m
            };
            // search direction: area decreases as the coordinate moves toward closure
            let base = p.targets.get(key).unwrap_or(f64::NAN);
            let (mut lo, mut hi) = (base - 1.2, base + 1.2);
            let decreasing = min_area(hi) < min_area(lo); // area falls as value rises?
            for _ in 0..40 {
                let mid = (lo + hi) / 2.0;
                let area_mid = min_area(mid);
                if (area_mid > target) == decreasing {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let v = (lo + hi) / 2.0;
            let got = min_area(v);
            table.push((
                p.ipa.to_string(),
                Json::Obj(vec![(key.name().to_string(), Json::Num(round3(v)))]),
            ));
            println!(
                "{} {} {}={} area {} (target {})",
                pad_end(name, 6),
                pad_end(p.ipa, 3),
                key.name(),
                fixed(v, 3),
                fixed(got, 3),
                fixed(target, 3)
            );
        }
        result.push((name.to_string(), Json::Obj(table)));
    }
    std::fs::write(&out, Json::Obj(result).stringify(1)).map_err(|e| format!("{out}: {e}"))?;
    Ok(())
}
