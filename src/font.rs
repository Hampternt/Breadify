//! The typefaces the printed page uses, embedded in the binary.
//!
//! Static instances only. A variable font file embeds its default instance
//! without complaint — feed `SpaceGrotesk[wght].ttf` to a PDF writer and the
//! page comes out Light with no diagnostic — and under this architecture it
//! would bite twice, because the measurement and the drawing must load the
//! same face or the positions stop matching the glyphs.
//!
//! All three families are SIL Open Font License 1.1; the licences ship in
//! `assets/fonts/`.

/// A face the page can set type in.
///
/// Archivo carries headings and anything that must be found at arm's length,
/// Space Grotesk the reading text, and IBM Plex Mono anything a machine
/// produced — quantities, codes, dates, identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    /// Customer names, department labels, badges.
    ArchivoExtraBold,
    /// The route number in the masthead.
    ArchivoBlack,
    /// Product names.
    SpaceGrotesk,
    /// Product names that need a little more weight.
    SpaceGroteskMedium,
    MonoRegular,
    MonoMedium,
    MonoSemiBold,
    MonoBold,
}

/// Every face, for tests and for embedding.
pub const ALL: [Face; 8] = [
    Face::ArchivoExtraBold,
    Face::ArchivoBlack,
    Face::SpaceGrotesk,
    Face::SpaceGroteskMedium,
    Face::MonoRegular,
    Face::MonoMedium,
    Face::MonoSemiBold,
    Face::MonoBold,
];

impl Face {
    /// The font file itself, compiled into the binary so a printed page never
    /// depends on what is installed.
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::ArchivoExtraBold => include_bytes!("../assets/fonts/Archivo-ExtraBold.ttf"),
            Self::ArchivoBlack => include_bytes!("../assets/fonts/Archivo-Black.ttf"),
            Self::SpaceGrotesk => include_bytes!("../assets/fonts/SpaceGrotesk-Regular.ttf"),
            Self::SpaceGroteskMedium => include_bytes!("../assets/fonts/SpaceGrotesk-Medium.ttf"),
            Self::MonoRegular => include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"),
            Self::MonoMedium => include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf"),
            Self::MonoSemiBold => include_bytes!("../assets/fonts/IBMPlexMono-SemiBold.ttf"),
            Self::MonoBold => include_bytes!("../assets/fonts/IBMPlexMono-Bold.ttf"),
        }
    }

    /// The family name the file declares.
    pub fn family(self) -> &'static str {
        match self {
            Self::ArchivoExtraBold | Self::ArchivoBlack => "Archivo",
            Self::SpaceGrotesk | Self::SpaceGroteskMedium => "Space Grotesk",
            _ => "IBM Plex Mono",
        }
    }

    /// The CSS weight the design asks for.
    pub fn weight(self) -> u16 {
        match self {
            Self::ArchivoExtraBold => 800,
            Self::ArchivoBlack => 900,
            Self::SpaceGrotesk | Self::MonoRegular => 400,
            Self::SpaceGroteskMedium | Self::MonoMedium => 500,
            Self::MonoSemiBold => 600,
            Self::MonoBold => 700,
        }
    }

    /// Parses the face for measuring.
    ///
    /// # Panics
    ///
    /// If the embedded file is not a readable font, which would be a broken
    /// build rather than a runtime condition.
    pub fn parsed(self) -> ttf_parser::Face<'static> {
        ttf_parser::Face::parse(self.bytes(), 0).unwrap_or_else(|error| {
            panic!("{} {} should parse: {error}", self.family(), self.weight())
        })
    }
}
