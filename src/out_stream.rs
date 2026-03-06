use std::{fs, io, process};

#[derive(Debug)]
pub enum OutStream {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    File(fs::File),
}

#[derive(Debug)]
pub struct Redirects {
    pub stdin: io::Stdin,
    pub stdout: OutStream,
    pub stderr: OutStream,
}

impl From<OutStream> for process::Stdio {
    fn from(val: OutStream) -> Self {
        match val {
            OutStream::Stdout(s) => process::Stdio::from(s),
            OutStream::Stderr(s) => process::Stdio::from(s),
            OutStream::File(s) => process::Stdio::from(s),
        }
    }
}

impl io::Write for OutStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(s) => s.write(buf),
            Self::Stderr(s) => s.write(buf),
            Self::File(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout(s) => s.flush(),
            Self::Stderr(s) => s.flush(),
            Self::File(s) => s.flush(),
        }
    }
}

impl From<io::Stderr> for OutStream {
    fn from(value: io::Stderr) -> Self {
        OutStream::Stderr(value)
    }
}

impl From<io::Stdout> for OutStream {
    fn from(value: io::Stdout) -> Self {
        OutStream::Stdout(value)
    }
}

impl From<fs::File> for OutStream {
    fn from(value: fs::File) -> Self {
        OutStream::File(value)
    }
}
