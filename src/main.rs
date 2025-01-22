use anyhow::Result;
use nix::sys::termios::{
    self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SpecialCharacterIndices, Termios,
};
use std::{io::Read, os::fd::AsFd};

struct Terminal {
    orig: Termios,
    curr: Termios,
}

impl Terminal {
    fn new() -> Self {
        let orig = termios::tcgetattr(std::io::stdin().as_fd()).unwrap();
        let curr = orig.clone();
        Self { orig, curr }
    }

    fn enable_raw_mode(&mut self, stdin: &std::io::Stdin) -> Result<()> {
        // Disable input flags
        // BRKINT, ICRNL, INPCK, ISTRIP, IXON
        termios::InputFlags::remove(&mut self.curr.input_flags, InputFlags::BRKINT);
        termios::InputFlags::remove(&mut self.curr.input_flags, InputFlags::ICRNL);
        termios::InputFlags::remove(&mut self.curr.input_flags, InputFlags::INPCK);
        termios::InputFlags::remove(&mut self.curr.input_flags, InputFlags::ISTRIP);
        termios::InputFlags::remove(&mut self.curr.input_flags, InputFlags::IXON);

        // Disable output flags
        termios::OutputFlags::remove(&mut self.curr.output_flags, OutputFlags::OPOST);

        // Set the control flags to 8 data bits, no parity, no stop bits.
        termios::ControlFlags::set(&mut self.curr.control_flags, ControlFlags::CS8, true);

        // Disable local flags
        termios::LocalFlags::remove(&mut self.curr.local_flags, LocalFlags::ECHO);
        termios::LocalFlags::remove(&mut self.curr.local_flags, LocalFlags::ICANON);
        termios::LocalFlags::remove(&mut self.curr.local_flags, LocalFlags::IEXTEN);
        termios::LocalFlags::remove(&mut self.curr.local_flags, LocalFlags::ISIG);

        // Set the minimum number of characters to read before returning from read()
        // and the timeout in deciseconds.
        self.curr.control_chars[SpecialCharacterIndices::VMIN as usize] = 0;
        self.curr.control_chars[SpecialCharacterIndices::VTIME as usize] = 1;

        // Apply the new settings
        termios::tcsetattr(stdin.as_fd(), termios::SetArg::TCSAFLUSH, &self.curr)?;

        Ok(())
    }

    fn disable_raw_mode(&mut self, stdin: &std::io::Stdin) -> Result<()> {
        termios::tcsetattr(stdin.as_fd(), termios::SetArg::TCSAFLUSH, &self.orig)?;
        Ok(())
    }
}

fn ctrl_key(c: char) -> u8 {
    c as u8 & 0x1f
}

fn editor_read_key(handle: &mut std::io::StdinLock<'_>) -> Result<u8> {
    let mut buf = [0u8; 1];
    handle.read(&mut buf)?;
    Ok(buf[0])
}

fn main() -> Result<()> {
    let mut terminal = Terminal::new();

    let stdin = std::io::stdin();
    terminal.enable_raw_mode(&stdin)?;

    let mut handle = stdin.lock();

    loop {
        match editor_read_key(&mut handle)? {
            c if c == ctrl_key('q') => break,
            c if c.is_ascii_control() => print!("{c}\r\n"),
            c => print!("{c} ('{}')\r\n", c as char),
        }
    }

    terminal.disable_raw_mode(&stdin)?;

    Ok(())
}
