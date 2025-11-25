use memory::Pod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
struct TagId([u8; 4]);

impl TagId {
    pub fn from_str(str: &str) -> Self {
        let bytes = str.as_bytes();

        if bytes.len() != 4 {
            panic!("Invalid tag ID bytes");
        }

        TagId([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let Some(open_bracket_pos) = name.find('[') else {
            return None;
        };

        let Some(close_bracket_pos) = name.find(']') else {
            return None;
        };

        if close_bracket_pos != open_bracket_pos + 5 {
            return None;
        }

        Some(TagId::from_str(
            &name[open_bracket_pos + 1..close_bracket_pos],
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
struct Suffix(u32);

impl Suffix {
    pub const IM08: Self = Self(0x3830_6d69); // "im08"
    pub const IM16: Self = Self(0x3631_6d69); // "im16"
    pub const SOUN: Self = Self(0x6e75_6f73); // "soun"
    pub const STLI: Self = Self(0x696c_7473); // "stli"
    pub const FLLI: Self = Self(0x696c_6c66); // "flli"
    pub const WEDE: Self = Self(0x6564_6577); // "wede"
    pub const RELI: Self = Self(0x696c_6572); // "reli"
    pub const EDLI: Self = Self(0x696c_6465); // "edli"
    pub const COLI: Self = Self(0x696c_6f63); // "coli"
    pub const TEFO: Self = Self(0x6f66_6574); // "tefo"
    pub const PLDE: Self = Self(0x6564_6c70); // "plde"
    pub const PREF: Self = Self(0x6665_7270); // "pref"
    pub const FILM: Self = Self(0x6d6c_6966); // "film"
    pub const UNDE: Self = Self(0x6564_6e75); // "unde"
    pub const LEVE: Self = Self(0x6576_656c); // "leve"

    pub fn from_name(name: &str) -> Option<Self> {
        let Some(dot_pos) = name.find('.') else {
            return None;
        };

        match name[dot_pos + 1..].to_lowercase().as_str() {
            "im08" | "gif" | "giff" => Some(Self::IM08),
            "im16" | "tga" | "targa" => Some(Self::IM16),
            "snd" | "sound" | "aif" | "aiff" | "ima" => Some(Self::SOUN),
            "txt" | "text" | "string" | "stli" => Some(Self::STLI),
            "flt" | "float" | "flli" => Some(Self::FLLI),
            "we" | "wep" | "wede" => Some(Self::WEDE),
            "rect" | "reli" | "rectlist" => Some(Self::RELI),
            "id" | "idli" | "idlist" => Some(Self::EDLI),
            "co" | "coli" | "color" | "colorlist" => Some(Self::COLI),
            "tefo" | "textformat" => Some(Self::TEFO),
            "plde" | "player" => Some(Self::PLDE),
            "pref" => Some(Self::PREF),
            "film" => Some(Self::FILM),
            "unde" | "unitdef" | "unit" => Some(Self::UNDE),
            "lvl" | "leve" | "level" => Some(Self::LEVE),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct Tag {
    _unknown: [u8; 0x20],
    suffix: Suffix,
    id: TagId,
}

impl Tag {
    pub fn get_info_from_file_name(&mut self, name: &str) -> bool {
        if name.is_empty() {
            panic!("File name cannot be empty");
        }

        if name.contains(".pak") || name.contains(".zip") {
            eprintln!(
                "\nFILE ALERT.  Skipping file.  (Zip files are not supported in the Local directory)\nOffending file:  \"{}\"\n",
                name
            );
            return false;
        }

        if !name.contains('[') {
            eprintln!(
                "\nFILE ALERT.  Skipping file, as no valid information was found in filename:  \"{}\"",
                name
            );
            return false;
        };

        let Some(tag_id) = TagId::from_name(name) else {
            eprintln!("\nFILE ALERT.  Could not find Tag ID in file:  {}", name);
            return false;
        };

        self.id = tag_id;

        let Some(suffix) = Suffix::from_name(name) else {
            eprintln!(
                "\nFILE ALERT.  Could not find a valid suffix in file:  {}",
                name
            );
            return false;
        };

        self.suffix = suffix;

        log::debug!(
            "get_info_from_file_name({}) = {:?} + {:?}",
            name,
            tag_id,
            suffix
        );

        true
    }
}

unsafe impl Pod for Tag {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_id_from_name() {
        assert_eq!(
            TagId::from_name("Attributions[cred].stli"),
            Some(TagId::from_str("cred"))
        );

        assert_eq!(TagId::from_name("Attributions[cred.stli"), None);
        assert_eq!(TagId::from_name("Attributions[credo].stli"), None);
        assert_eq!(TagId::from_name("Attributions cred].stli"), None);
    }

    #[test]
    fn test_suffix_from_name() {
        fn test(name: &str, expected: Suffix) {
            assert_eq!(Suffix::from_name(name), Some(expected));
        }

        test("Colors[gaco].coli", Suffix::COLI);
        test("Demo 01[de01].film", Suffix::FILM);
        test("Game[gafl].flli", Suffix::FLLI);
        test("Editor[edit].idli", Suffix::EDLI);
        test("Baccula Shields IA[BASH].gif", Suffix::IM08);
        test("Background[back].TGA", Suffix::IM16);
        test("Level 01[le01].leve", Suffix::LEVE);
        test("Player 1[pl01].plde", Suffix::PLDE);
        test("Preferences[pref].pref", Suffix::PREF);
        test("Rects[inre].reli", Suffix::RELI);
        test("Accuracy Bonus[acbo].IMA", Suffix::SOUN);
        test("Music 3[mu03].aif", Suffix::SOUN);
        test("Attributions[cred].stli", Suffix::STLI);
        test("Briefing Normal[brno].tefo", Suffix::TEFO);
        test("Bacta Gun - Bullet[bagb].unde", Suffix::UNDE);
        test("Air - Bacta Gun[aibg].wede", Suffix::WEDE);
    }
}
