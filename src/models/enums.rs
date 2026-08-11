//! API enums. Open-vocabulary fields (`gear`, `format`, `license`, `size`) are
//! `#[non_exhaustive]` with an `Other(String)` catch-all so unknown values never fail a
//! response. Sort enums are inputs we send, with the exact wire strings the API expects.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! open_enum {
    (
        $(#[$m:meta])*
        $name:ident { $( $variant:ident => $wire:literal ),+ $(,)? }
    ) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        pub enum $name {
            $( #[doc = $wire] $variant, )+
            /// A value not recognized by this version of the SDK.
            Other(String),
        }

        impl $name {
            /// The wire string for this value.
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
        Amp => "amp",
        AmpCab => "amp-cab",
        FullRig => "full-rig",
        Pedal => "pedal",
        Outboard => "outboard",
        Cab => "cab",
        Space => "space",
        Experimental => "experimental",
        Ir => "ir",
    }
);

open_enum!(
    /// Model file format. Renamed from `platform` by the API in 2026; the value set is
    /// unchanged.
    Format {
        Nam => "nam",
        Ir => "ir",
        AidaX => "aida-x",
        AaSnapshot => "aa-snapshot",
        Proteus => "proteus",
    }
);

open_enum!(
    /// Tone license.
    License {
        T3k => "t3k",
        CcBy => "cc-by",
        CcBySa => "cc-by-sa",
        CcByNc => "cc-by-nc",
        CcByNcSa => "cc-by-nc-sa",
        CcByNd => "cc-by-nd",
        CcByNcNd => "cc-by-nc-nd",
        Cco => "cco",
    }
);

open_enum!(
    /// Model size class.
    Size {
        Standard => "standard",
        Lite => "lite",
        Feather => "feather",
        Nano => "nano",
        Custom => "custom",
    }
);

open_enum!(
    /// Neural model architecture version. `custom` covers user-supplied architectures.
    ArchitectureVersion {
        V1 => "1",
        V2 => "2",
        Custom => "custom",
    }
);

/// Sort order for tone listing/search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToneSort {
    BestMatch,
    Newest,
    Oldest,
    Trending,
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
    Tones,
    Downloads,
    Favorites,
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
