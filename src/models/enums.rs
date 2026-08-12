//! API enums. Open-vocabulary fields (`gear`, `format`, `license`, `size`,
//! `architecture_version`) are `#[non_exhaustive]` with an `Other(String)` catch-all so
//! unknown values never fail a response. Sort enums are inputs we send, with the exact
//! wire strings the API expects.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! open_enum {
    (
        $(#[$m:meta])*
        $name:ident { $( $(#[doc = $doc:literal])+ $variant:ident => $wire:literal ),+ $(,)? }
    ) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[doc = $doc])+ $variant, )+
            /// A value not recognized by this version of the SDK.
            Other(String),
        }

        impl $name {
            /// The exact string this value takes on the wire, e.g. `"amp-cab"`.
            ///
            /// Every variant round-trips: parsing this string yields the same value back,
            /// including unrecognised ones held in `Other`.
            pub fn as_str(&self) -> &str {
                match self {
                    $( $name::$variant => $wire, )+
                    $name::Other(s) => s.as_str(),
                }
            }

            fn from_wire(s: &str) -> Self {
                match s {
                    $( $wire => $name::$variant, )+
                    other => $name::Other(other.to_string()),
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                ser.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
                let s = String::deserialize(de)?;
                Ok($name::from_wire(&s))
            }
        }
    };
}

open_enum!(
    /// Gear category.
    ///
    /// Two values are being retired by the API:
    /// - `full-rig` is deprecated — responses now emit `amp-cab` instead, though `full-rig`
    ///   is still accepted as search input.
    /// - `ir` is being retired as a gear; filter on [`Format::Ir`] instead.
    Gear {
        /// A guitar or bass amplifier, captured without a cabinet.
        Amp => "amp",
        /// An amplifier and cabinet captured together as one signal chain.
        AmpCab => "amp-cab",
        /// A whole signal chain, pedals included.
        ///
        /// Being retired: responses now use [`Gear::AmpCab`] instead, though the API still
        /// accepts `full-rig` as a search filter.
        FullRig => "full-rig",
        /// An overdrive, distortion, fuzz or other stompbox.
        Pedal => "pedal",
        /// Studio outboard — preamps, compressors, EQs, console channels.
        Outboard => "outboard",
        /// A speaker cabinet on its own, without an amp.
        Cab => "cab",
        /// A room, hall or other acoustic space.
        Space => "space",
        /// Captures that do not fit the other categories.
        Experimental => "experimental",
        /// Impulse responses.
        ///
        /// Being retired as a gear category — filter on [`Format::Ir`] instead, which is
        /// where format now lives.
        Ir => "ir",
    }
);

open_enum!(
    /// Model file format. Renamed from `platform` by the API in 2026; the value set is
    /// unchanged.
    Format {
        /// Neural Amp Modeler. Load with a NAM player such as `nam-rs`.
        Nam => "nam",
        /// An impulse response: a convolution file, not a neural capture.
        Ir => "ir",
        /// AIDA-X, the LV2/plugin neural format.
        AidaX => "aida-x",
        /// An Amp Academy pedal snapshot.
        AaSnapshot => "aa-snapshot",
        /// A GuitarML Proteus capture.
        Proteus => "proteus",
    }
);

open_enum!(
    /// Tone license.
    License {
        /// TONE3000's own licence. Read the site terms before redistributing.
        T3k => "t3k",
        /// Creative Commons Attribution: reuse freely, credit the creator.
        CcBy => "cc-by",
        /// CC Attribution-ShareAlike: credit, and share derivatives on the same terms.
        CcBySa => "cc-by-sa",
        /// CC Attribution-NonCommercial: credit, no commercial use.
        CcByNc => "cc-by-nc",
        /// CC Attribution-NonCommercial-ShareAlike.
        CcByNcSa => "cc-by-nc-sa",
        /// CC Attribution-NoDerivatives: credit, redistribute unmodified only.
        CcByNd => "cc-by-nd",
        /// CC Attribution-NonCommercial-NoDerivatives.
        CcByNcNd => "cc-by-nc-nd",
        /// CC0: dedicated to the public domain.
        Cco => "cco",
    }
);

open_enum!(
    /// Model size class.
    Size {
        /// Full quality, highest CPU cost.
        Standard => "standard",
        /// Reduced quality for less CPU than [`Size::Standard`].
        Lite => "lite",
        /// Smaller again than [`Size::Lite`].
        Feather => "feather",
        /// The smallest standard size.
        Nano => "nano",
        /// A creator-defined size outside the standard ladder.
        Custom => "custom",
    }
);

open_enum!(
    /// Neural model architecture version. `custom` covers user-supplied architectures.
    ArchitectureVersion {
        /// The original NAM architecture. What `Client::models` returns unless you ask
        /// for another.
        V1 => "1",
        /// The second-generation NAM architecture.
        V2 => "2",
        /// A user-supplied architecture rather than a standard one.
        Custom => "custom",
    }
);

/// Sort order for tone listing/search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToneSort {
    /// Relevance to the search query. Meaningless without one.
    BestMatch,
    /// Most recently published first. The best way to see new vocabulary and new gear.
    Newest,
    /// Oldest first.
    Oldest,
    /// Currently popular — recent downloads and favourites, not all-time totals.
    Trending,
    /// Most downloaded ever. The steadiest choice for a "top tones" landing page.
    DownloadsAllTime,
}

impl ToneSort {
    /// The query-string value the API expects.
    pub fn as_str(self) -> &'static str {
        match self {
            ToneSort::BestMatch => "best-match",
            ToneSort::Newest => "newest",
            ToneSort::Oldest => "oldest",
            ToneSort::Trending => "trending",
            ToneSort::DownloadsAllTime => "downloads-all-time",
        }
    }
}

/// Sort order for the public user list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UserSort {
    /// Most capture projects published.
    Tones,
    /// Most downloads received across everything they published.
    Downloads,
    /// Most favourites received.
    Favorites,
    /// Most individual model files published.
    Models,
}

impl UserSort {
    /// The query-string value the API expects.
    pub fn as_str(self) -> &'static str {
        match self {
            UserSort::Tones => "tones",
            UserSort::Downloads => "downloads",
            UserSort::Favorites => "favorites",
            UserSort::Models => "models",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_value_round_trips() {
        let g: Gear = serde_json::from_str("\"amp\"").unwrap();
        assert_eq!(g, Gear::Amp);
        assert_eq!(serde_json::to_string(&g).unwrap(), "\"amp\"");
        assert_eq!(g.as_str(), "amp");
    }

    #[test]
    fn unknown_value_falls_back_to_other() {
        let f: Format = serde_json::from_str("\"future-format\"").unwrap();
        assert_eq!(f, Format::Other("future-format".into()));
        assert_eq!(serde_json::to_string(&f).unwrap(), "\"future-format\"");
    }

    #[test]
    fn gear_covers_current_api_vocabulary() {
        for wire in [
            "amp",
            "amp-cab",
            "full-rig",
            "pedal",
            "outboard",
            "cab",
            "space",
            "experimental",
            "ir",
        ] {
            let g: Gear = serde_json::from_str(&format!("\"{wire}\"")).unwrap();
            assert!(
                !matches!(g, Gear::Other(_)),
                "{wire} should be a known Gear variant, got {g:?}"
            );
            assert_eq!(g.as_str(), wire);
        }
    }

    #[test]
    fn sort_wire_values() {
        assert_eq!(ToneSort::DownloadsAllTime.as_str(), "downloads-all-time");
        assert_eq!(UserSort::Favorites.as_str(), "favorites");
    }
}
