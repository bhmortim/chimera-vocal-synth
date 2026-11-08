//! How finely the human voice is simulated, and how much anatomy its airway carries.
//!
//! [`Detail::STANDARD`] is the calibrated model: the default voice and every golden test use
//! it, bit for bit. The other settings refine it, for listening and comparison:
//!
//! - [`Resolution`] raises the internal rate to ×1.5 or ×2 the standard's (96 kHz at 48 kHz
//!   out). The airway's sections shorten with the rate, since the FDTD's stability limit sets
//!   their length (0.39 → 0.26 → 0.20 cm at 48 kHz out), and the area function's measuring
//!   grid gets as many more lines, so the finer sections see finer geometry.
//! - `anatomy` adds the side cavities the standard model folds together or leaves out: left
//!   and right piriform fossae instead of one combined branch, the valleculae, the laryngeal
//!   ventricles, and the second maxillary, sphenoid and frontal sinuses.
//!
//! The vowel calibration was made at the standard detail and is not refitted: the finer
//! models keep its formants within a few percent (`tests/detail.rs` holds them to that).

use crate::math::{js_max, js_round};
use serde::{Deserialize, Serialize};

/// The internal rate, and the measuring grid's density, relative to the standard model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Resolution {
    /// ×1: 2× oversampling at 48 kHz out, sections ≈ 0.39 cm
    #[default]
    Standard,
    /// ×1.5: 3× oversampling, sections ≈ 0.26 cm
    High,
    /// ×2: 4× oversampling, sections ≈ 0.20 cm
    Ultra,
}

impl Resolution {
    pub const ALL: [Resolution; 3] = [Resolution::Standard, Resolution::High, Resolution::Ultra];

    pub fn factor(self) -> f64 {
        match self {
            Resolution::Standard => 1.0,
            Resolution::High => 1.5,
            Resolution::Ultra => 2.0,
        }
    }

    /// The C ABI's encoding: 0 standard, 1 high, 2 ultra (anything else: standard).
    pub fn from_level(level: u32) -> Resolution {
        match level {
            1 => Resolution::High,
            2 => Resolution::Ultra,
            _ => Resolution::Standard,
        }
    }

    pub fn level(self) -> u32 {
        match self {
            Resolution::Standard => 0,
            Resolution::High => 1,
            Resolution::Ultra => 2,
        }
    }
}

/// How finely the human voice is simulated (see the module docs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Detail {
    pub resolution: Resolution,
    /// the extra side cavities: piriform fossae left and right, valleculae, laryngeal
    /// ventricles, and three more paranasal sinuses
    pub anatomy: bool,
}

impl Detail {
    pub const STANDARD: Detail = Detail { resolution: Resolution::Standard, anatomy: false };

    /// Every detail, the standard first (the order of [`Detail::index`]).
    pub const ALL: [Detail; 6] = [
        Detail { resolution: Resolution::Standard, anatomy: false },
        Detail { resolution: Resolution::Standard, anatomy: true },
        Detail { resolution: Resolution::High, anatomy: false },
        Detail { resolution: Resolution::High, anatomy: true },
        Detail { resolution: Resolution::Ultra, anatomy: false },
        Detail { resolution: Resolution::Ultra, anatomy: true },
    ];

    /// Position in [`Detail::ALL`].
    pub fn index(self) -> usize {
        2 * self.resolution.level() as usize + usize::from(self.anatomy)
    }

    /// "standard", "standard-anatomy", "high", …, "ultra-anatomy": the CLI's names, and the
    /// suffix of a detail's calibrated tables (`vowelTargets.<name>.generated.json`).
    pub fn name(self) -> &'static str {
        ["standard", "standard-anatomy", "high", "high-anatomy", "ultra", "ultra-anatomy"][self.index()]
    }

    pub fn from_name(name: &str) -> Option<Detail> {
        Detail::ALL.into_iter().find(|d| d.name() == name)
    }

    /// For `#[serde(skip_serializing_if)]`: options at the standard detail serialize as before.
    pub fn is_standard(d: &Detail) -> bool {
        *d == Detail::STANDARD
    }

    /// Oversampling at an output rate: the standard's (2× up to 60 kHz, 1× above) times the
    /// resolution's factor, rounded.
    pub fn oversample(self, fs_out: f64) -> usize {
        let base = if fs_out > 60000.0 { 1.0 } else { 2.0 };
        js_max(1.0, js_round(base * self.resolution.factor())) as usize
    }
}
