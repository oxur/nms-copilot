use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The four galaxy type classifications that affect planet generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GalaxyType {
    Norm,
    Lush,
    Harsh,
    Empty,
}

impl fmt::Display for GalaxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Norm => write!(f, "Normal"),
            Self::Lush => write!(f, "Lush"),
            Self::Harsh => write!(f, "Harsh"),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

/// Error returned when parsing a galaxy type string fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyTypeParseError(pub String);

impl fmt::Display for GalaxyTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown galaxy type: {}", self.0)
    }
}

impl std::error::Error for GalaxyTypeParseError {}

impl FromStr for GalaxyType {
    type Err = GalaxyTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "norm" | "normal" => Ok(Self::Norm),
            "lush" => Ok(Self::Lush),
            "harsh" => Ok(Self::Harsh),
            "empty" => Ok(Self::Empty),
            _ => Err(GalaxyTypeParseError(s.to_string())),
        }
    }
}

/// One of the 256 galaxies in No Man's Sky.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Galaxy {
    pub index: u8,
    pub name: &'static str,
    pub galaxy_type: GalaxyType,
}

impl Galaxy {
    /// Lookup galaxy by index (0-255). Always returns a valid Galaxy.
    pub fn by_index(index: u8) -> Self {
        let (name, galaxy_type) = GALAXIES[index as usize];
        Self {
            index,
            name,
            galaxy_type,
        }
    }
}

impl fmt::Display for Galaxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

use GalaxyType::*;

/// All 256 galaxies: (name, type).
///
/// Type pattern (0-indexed, derived from NMS wiki):
/// - Harsh (26): 2, 14, 22, 34, 42, 54, 62, 74, 82, 94, 102, 114, 122, 134,
///   142, 154, 162, 174, 182, 194, 202, 214, 222, 234, 242, 254
/// - Empty (26): 6, 11, 26, 31, 46, 51, 66, 71, 86, 91, 106, 111, 126, 131,
///   146, 151, 166, 171, 186, 191, 206, 211, 226, 231, 246, 251
/// - Lush (25): 9, 18, 29, 38, 49, 58, 69, 78, 89, 98, 109, 118, 129, 138,
///   149, 158, 169, 178, 189, 198, 209, 218, 229, 238, 249
/// - Norm (179): everything else
static GALAXIES: [(&str, GalaxyType); 256] = [
    ("Euclid", Norm),              // 0
    ("Hilbert Dimension", Norm),   // 1
    ("Calypso", Harsh),            // 2
    ("Hesperius Dimension", Norm), // 3
    ("Hyades", Norm),              // 4
    ("Ickjamatew", Norm),          // 5
    ("Budullangr", Empty),         // 6
    ("Kikolgallr", Norm),          // 7
    ("Eltiensleen", Norm),         // 8
    ("Eissentam", Lush),           // 9
    ("Elkupalos", Norm),           // 10
    ("Aptarkaba", Empty),          // 11
    ("Ontiniangp", Norm),          // 12
    ("Odiwagiri", Norm),           // 13
    ("Ogtialabi", Harsh),          // 14
    ("Muhacksonto", Norm),         // 15
    ("Hitonskyer", Norm),          // 16
    ("Rerasmutul", Norm),          // 17
    ("Isdoraijung", Lush),         // 18
    ("Doctinawyra", Norm),         // 19
    ("Loychazinq", Norm),          // 20
    ("Zukasizawa", Norm),          // 21
    ("Ekwathore", Harsh),          // 22
    ("Yeberhahne", Norm),          // 23
    ("Twerbetek", Norm),           // 24
    ("Sivarates", Norm),           // 25
    ("Eajerandal", Empty),         // 26
    ("Aldukesci", Norm),           // 27
    ("Wotyarogii", Norm),          // 28
    ("Sudzerbal", Lush),           // 29
    ("Maupenzhay", Norm),          // 30
    ("Sugueziume", Empty),         // 31
    ("Brogoweldian", Norm),        // 32
    ("Ehbogdenbu", Norm),          // 33
    ("Ijsenufryos", Harsh),        // 34
    ("Nipikulha", Norm),           // 35
    ("Autsurabin", Norm),          // 36
    ("Lusontrygiamh", Norm),       // 37
    ("Rewmanawa", Lush),           // 38
    ("Ethiophodhe", Norm),         // 39
    ("Urastrykle", Norm),          // 40
    ("Xobeurindj", Norm),          // 41
    ("Oniijialdu", Harsh),         // 42
    ("Wucetosucc", Norm),          // 43
    ("Ebyeloof", Norm),            // 44
    ("Odyavanta", Norm),           // 45
    ("Milekistri", Empty),         // 46
    ("Waferganh", Norm),           // 47
    ("Agnusopwit", Norm),          // 48
    ("Teyaypilny", Lush),          // 49
    ("Zalienkosm", Norm),          // 50
    ("Ladgudiraf", Empty),         // 51
    ("Mushonponte", Norm),         // 52
    ("Amsentisz", Norm),           // 53
    ("Fladiselm", Harsh),          // 54
    ("Laanawemb", Norm),           // 55
    ("Ilkerloor", Norm),           // 56
    ("Davanossi", Norm),           // 57
    ("Ploehrliou", Lush),          // 58
    ("Corpinyaya", Norm),          // 59
    ("Leckandmeram", Norm),        // 60
    ("Quulngais", Norm),           // 61
    ("Nokokipsechl", Harsh),       // 62
    ("Rinblodesa", Norm),          // 63
    ("Loydporpen", Norm),          // 64
    ("Ibtrevskip", Norm),          // 65
    ("Elkowaldb", Empty),          // 66
    ("Heholhofsko", Norm),         // 67
    ("Yebrilowisod", Norm),        // 68
    ("Husalvangewi", Lush),        // 69
    ("Ovna'uesed", Norm),          // 70
    ("Bahibusey", Empty),          // 71
    ("Nuybeliaure", Norm),         // 72
    ("Doshawchuc", Norm),          // 73
    ("Ruckinarkh", Harsh),         // 74
    ("Thorettac", Norm),           // 75
    ("Nuponoparau", Norm),         // 76
    ("Moglaschil", Norm),          // 77
    ("Uiweupose", Lush),           // 78
    ("Nasmilete", Norm),           // 79
    ("Ekdaluskin", Norm),          // 80
    ("Hakapanasy", Norm),          // 81
    ("Dimonimba", Harsh),          // 82
    ("Cajaccari", Norm),           // 83
    ("Olonerovo", Norm),           // 84
    ("Umlanswick", Norm),          // 85
    ("Henayliszm", Empty),         // 86
    ("Utzenmate", Norm),           // 87
    ("Umirpaiya", Norm),           // 88
    ("Paholiang", Lush),           // 89
    ("Iaereznika", Norm),          // 90
    ("Yudukagath", Empty),         // 91
    ("Boealalosnj", Norm),         // 92
    ("Yaevarcko", Norm),           // 93
    ("Coellosipp", Harsh),         // 94
    ("Wayndohalou", Norm),         // 95
    ("Smoduraykl", Norm),          // 96
    ("Apmaneessu", Norm),          // 97
    ("Hicanpaav", Lush),           // 98
    ("Akvasanta", Norm),           // 99
    ("Tuychelisaor", Norm),        // 100
    ("Rivskimbe", Norm),           // 101
    ("Daksanquix", Harsh),         // 102
    ("Kissonlin", Norm),           // 103
    ("Aediabiel", Norm),           // 104
    ("Ulosaginyik", Norm),         // 105
    ("Roclaytonycar", Empty),      // 106
    ("Kichiaroa", Norm),           // 107
    ("Irceauffey", Norm),          // 108
    ("Nudquathsenfe", Lush),       // 109
    ("Getaizakaal", Norm),         // 110
    ("Hansolmien", Empty),         // 111
    ("Bloytisagra", Norm),         // 112
    ("Ladsenlay", Norm),           // 113
    ("Luyugoslasr", Harsh),        // 114
    ("Ubredhatk", Norm),           // 115
    ("Cidoniana", Norm),           // 116
    ("Jasinessa", Norm),           // 117
    ("Torweierf", Lush),           // 118
    ("Saffneckm", Norm),           // 119
    ("Thnistner", Norm),           // 120
    ("Dotusingg", Norm),           // 121
    ("Luleukous", Harsh),          // 122
    ("Jelmandan", Norm),           // 123
    ("Otimanaso", Norm),           // 124
    ("Enjaxusanto", Norm),         // 125
    ("Sezviktorew", Empty),        // 126
    ("Zikehpm", Norm),             // 127
    ("Bephembah", Norm),           // 128
    ("Broomerrai", Lush),          // 129
    ("Meximicka", Norm),           // 130
    ("Venessika", Empty),          // 131
    ("Gaiteseling", Norm),         // 132
    ("Zosakasiro", Norm),          // 133
    ("Drajayanes", Harsh),         // 134
    ("Ooibekuar", Norm),           // 135
    ("Urckiansi", Norm),           // 136
    ("Dozivadido", Norm),          // 137
    ("Emiekereks", Lush),          // 138
    ("Meykinunukur", Norm),        // 139
    ("Kimycuristh", Norm),         // 140
    ("Roansfien", Norm),           // 141
    ("Isgarmeso", Harsh),          // 142
    ("Daitibeli", Norm),           // 143
    ("Gucuttarik", Norm),          // 144
    ("Enlaythie", Norm),           // 145
    ("Drewweste", Empty),          // 146
    ("Akbulkabi", Norm),           // 147
    ("Homskiw", Norm),             // 148
    ("Zavainlani", Lush),          // 149
    ("Jewijkmas", Norm),           // 150
    ("Itlhotagra", Empty),         // 151
    ("Podalicess", Norm),          // 152
    ("Hiviusauer", Norm),          // 153
    ("Halsebenk", Harsh),          // 154
    ("Puikitoac", Norm),           // 155
    ("Gaybakuaria", Norm),         // 156
    ("Grbodubhe", Norm),           // 157
    ("Rycempler", Lush),           // 158
    ("Indjalala", Norm),           // 159
    ("Fontenikk", Norm),           // 160
    ("Pasycihelwhee", Norm),       // 161
    ("Ikbaksmit", Harsh),          // 162
    ("Telicianses", Norm),         // 163
    ("Oyleyzhan", Norm),           // 164
    ("Uagerosat", Norm),           // 165
    ("Impoxectin", Empty),         // 166
    ("Twoodmand", Norm),           // 167
    ("Hilfsesorbs", Norm),         // 168
    ("Ezdaranit", Lush),           // 169
    ("Wiensanshe", Norm),          // 170
    ("Ewheelonc", Empty),          // 171
    ("Litzmantufa", Norm),         // 172
    ("Emarmatosi", Norm),          // 173
    ("Mufimbomacvi", Harsh),       // 174
    ("Wongquarum", Norm),          // 175
    ("Hapirajua", Norm),           // 176
    ("Igbinduina", Norm),          // 177
    ("Wepaitvas", Lush),           // 178
    ("Sthatigudi", Norm),          // 179
    ("Yekathsebehn", Norm),        // 180
    ("Ebedeagurst", Norm),         // 181
    ("Nolisonia", Harsh),          // 182
    ("Ulexovitab", Norm),          // 183
    ("Iodhinxois", Norm),          // 184
    ("Irroswitzs", Norm),          // 185
    ("Bifredait", Empty),          // 186
    ("Beiraghedwe", Norm),         // 187
    ("Yeonatlak", Norm),           // 188
    ("Cugnatachh", Lush),          // 189
    ("Nozoryenki", Norm),          // 190
    ("Ebralduri", Empty),          // 191
    ("Evcickcandj", Norm),         // 192
    ("Ziybosswin", Norm),          // 193
    ("Heperclait", Harsh),         // 194
    ("Sugiuniam", Norm),           // 195
    ("Aaseertush", Norm),          // 196
    ("Uglyestemaa", Norm),         // 197
    ("Horeroedsh", Lush),          // 198
    ("Drundemiso", Norm),          // 199
    ("Ityanianat", Norm),          // 200
    ("Purneyrine", Norm),          // 201
    ("Dokiessmat", Harsh),         // 202
    ("Nupiacheh", Norm),           // 203
    ("Dihewsonj", Norm),           // 204
    ("Rudrailhik", Norm),          // 205
    ("Tweretnort", Empty),         // 206
    ("Snatreetze", Norm),          // 207
    ("Iwundaracos", Norm),         // 208
    ("Digarlewena", Lush),         // 209
    ("Erquagsta", Norm),           // 210
    ("Logovoloin", Empty),         // 211
    ("Boyaghosganh", Norm),        // 212
    ("Kuolungau", Norm),           // 213
    ("Pehneldept", Harsh),         // 214
    ("Yevettiiqidcon", Norm),      // 215
    ("Sahliacabru", Norm),         // 216
    ("Noggalterpor", Norm),        // 217
    ("Chmageaki", Lush),           // 218
    ("Veticueca", Norm),           // 219
    ("Vittesbursul", Norm),        // 220
    ("Nootanore", Norm),           // 221
    ("Innebdjerah", Harsh),        // 222
    ("Kisvarcini", Norm),          // 223
    ("Cuzcogipper", Norm),         // 224
    ("Pamanhermonsu", Norm),       // 225
    ("Brotoghek", Empty),          // 226
    ("Mibittara", Norm),           // 227
    ("Huruahili", Norm),           // 228
    ("Raldwicarn", Lush),          // 229
    ("Ezdartlic", Norm),           // 230
    ("Badesclema", Empty),         // 231
    ("Isenkeyan", Norm),           // 232
    ("Iadoitesu", Norm),           // 233
    ("Yagrovoisi", Harsh),         // 234
    ("Ewcomechio", Norm),          // 235
    ("Inunnunnoda", Norm),         // 236
    ("Dischiutun", Norm),          // 237
    ("Yuwarugha", Lush),           // 238
    ("Ialmendra", Norm),           // 239
    ("Reponudrle", Norm),          // 240
    ("Rinjanagrbo", Norm),         // 241
    ("Zeziceloh", Harsh),          // 242
    ("Oeileutasc", Norm),          // 243
    ("Zicniijinis", Norm),         // 244
    ("Dugnowarilda", Norm),        // 245
    ("Neuxoisan", Empty),          // 246
    ("Ilmenhorn", Norm),           // 247
    ("Rukwatsuku", Norm),          // 248
    ("Nepitzaspru", Lush),         // 249
    ("Chcehoemig", Norm),          // 250
    ("Haffneyrin", Empty),         // 251
    ("Uliciawai", Norm),           // 252
    ("Tuhgrespod", Norm),          // 253
    ("Iousongola", Harsh),         // 254
    ("Odyalutai", Norm),           // 255
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclid_is_index_zero() {
        let g = Galaxy::by_index(0);
        assert_eq!(g.index, 0);
        assert_eq!(g.name, "Euclid");
        assert_eq!(g.galaxy_type, GalaxyType::Norm);
    }

    #[test]
    fn calypso_is_harsh() {
        let g = Galaxy::by_index(2);
        assert_eq!(g.name, "Calypso");
        assert_eq!(g.galaxy_type, GalaxyType::Harsh);
    }

    #[test]
    fn eissentam_is_lush() {
        let g = Galaxy::by_index(9);
        assert_eq!(g.name, "Eissentam");
        assert_eq!(g.galaxy_type, GalaxyType::Lush);
    }

    #[test]
    fn budullangr_is_empty() {
        let g = Galaxy::by_index(6);
        assert_eq!(g.name, "Budullangr");
        assert_eq!(g.galaxy_type, GalaxyType::Empty);
    }

    #[test]
    fn all_256_valid() {
        for i in 0..=255u8 {
            let g = Galaxy::by_index(i);
            assert_eq!(g.index, i);
            assert!(!g.name.is_empty());
        }
    }

    #[test]
    fn type_counts() {
        let mut norm = 0u16;
        let mut lush = 0u16;
        let mut harsh = 0u16;
        let mut empty = 0u16;
        for i in 0..=255u8 {
            match Galaxy::by_index(i).galaxy_type {
                GalaxyType::Norm => norm += 1,
                GalaxyType::Lush => lush += 1,
                GalaxyType::Harsh => harsh += 1,
                GalaxyType::Empty => empty += 1,
            }
        }
        assert_eq!(lush, 25);
        assert_eq!(harsh, 26);
        assert_eq!(empty, 26);
        assert_eq!(norm, 179);
    }

    #[test]
    fn last_galaxy() {
        let g = Galaxy::by_index(255);
        assert_eq!(g.name, "Odyalutai");
    }

    #[test]
    fn galaxy_display() {
        let g = Galaxy::by_index(0);
        assert_eq!(format!("{g}"), "Euclid");
    }

    #[test]
    fn galaxy_type_display_fromstr_roundtrip() {
        for gt in [
            GalaxyType::Norm,
            GalaxyType::Lush,
            GalaxyType::Harsh,
            GalaxyType::Empty,
        ] {
            let s = gt.to_string();
            let parsed: GalaxyType = s.parse().unwrap();
            assert_eq!(gt, parsed);
        }
    }

    #[test]
    fn galaxy_type_alternate_names() {
        assert_eq!("norm".parse::<GalaxyType>().unwrap(), GalaxyType::Norm);
        assert_eq!("normal".parse::<GalaxyType>().unwrap(), GalaxyType::Norm);
    }
}
