use anyhow::Result;
use portable_pty::MasterPty;
use std::io::{Read, Write};

/// Handles I/O operations for the PTY
pub struct PtyIo {
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl PtyIo {
    pub fn new(master: &dyn MasterPty) -> Result<Self> {
        let reader = master.try_clone_reader()?;
        let writer = master.take_writer()?;
        Ok(Self { reader, writer })
    }

    /// Read available data from PTY (non-blocking read attempt)
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }

    /// Write data to PTY
    pub fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.writer.write(data)
    }

    /// Flush the writer
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
