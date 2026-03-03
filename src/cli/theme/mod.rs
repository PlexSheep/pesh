pub mod posix;

use posix::PosixTheme;

use dialoguer::theme::ColorfulTheme;

#[allow(clippy::large_enum_variant)]
pub enum Theme {
    Fancy(ColorfulTheme),
    Posix(PosixTheme),
}

impl Theme {
    pub fn downcast(&self) -> &dyn dialoguer::theme::Theme {
        match self {
            Self::Fancy(t) => t,
            Self::Posix(t) => t,
        }
    }
}
