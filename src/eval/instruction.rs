use strum_macros::{Display, EnumCount, EnumIter, EnumString};

#[derive(Debug, Clone, Hash)]
pub enum Instruction {
    Builtin(BuiltinInstruction),
    Extern(Vec<String>),
}

#[allow(nonstandard_style)] // these are literally the builtin commands
#[derive(Debug, Clone, Hash, Display, EnumCount, EnumIter, EnumString)]
pub enum BuiltinInstruction {
    exit,
    pwd,
    cd(Option<std::path::PathBuf>),
}
