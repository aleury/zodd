use anyhow::Result;
use nix::sys::termios::{
    self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SpecialCharacterIndices,
};
use std::{io::Read, os::fd::AsFd};

static mut ORIG_TERMIOS: Option<termios::Termios> = None;

fn enable_raw_mode(stdin: &std::io::Stdin) -> Result<()> {
    let orig = termios::tcgetattr(stdin.as_fd())?;
    let mut raw = orig.clone();
    unsafe {
        ORIG_TERMIOS = Some(orig.clone());
    }

    // Disable input flags
    // BRKINT, ICRNL, INPCK, ISTRIP, IXON
    termios::InputFlags::remove(&mut raw.input_flags, InputFlags::BRKINT);
    termios::InputFlags::remove(&mut raw.input_flags, InputFlags::ICRNL);
    termios::InputFlags::remove(&mut raw.input_flags, InputFlags::INPCK);
    termios::InputFlags::remove(&mut raw.input_flags, InputFlags::ISTRIP);
    termios::InputFlags::remove(&mut raw.input_flags, InputFlags::IXON);

    // Disable output flags
    termios::OutputFlags::remove(&mut raw.output_flags, OutputFlags::OPOST);

    // Set the control flags to 8 data bits, no parity, no stop bits.
    termios::ControlFlags::set(&mut raw.control_flags, ControlFlags::CS8, true);

    // Disable local flags
    termios::LocalFlags::remove(&mut raw.local_flags, LocalFlags::ECHO);
    termios::LocalFlags::remove(&mut raw.local_flags, LocalFlags::ICANON);
    termios::LocalFlags::remove(&mut raw.local_flags, LocalFlags::IEXTEN);
    termios::LocalFlags::remove(&mut raw.local_flags, LocalFlags::ISIG);

    // Set the minimum number of characters to read before returning from read()
    // and the timeout in deciseconds.
    raw.control_chars[SpecialCharacterIndices::VMIN as usize] = 0;
    raw.control_chars[SpecialCharacterIndices::VTIME as usize] = 1;

    // Apply the new settings
    termios::tcsetattr(stdin.as_fd(), termios::SetArg::TCSAFLUSH, &raw)?;

    Ok(())
}

fn disable_raw_mode(stdin: &std::io::Stdin) -> Result<()> {
    let orig = unsafe { ORIG_TERMIOS.clone() };
    if let Some(orig) = orig {
        termios::tcsetattr(stdin.as_fd(), termios::SetArg::TCSAFLUSH, &orig)?;
    }
    Ok(())
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
    let stdin = std::io::stdin();
    enable_raw_mode(&stdin)?;

    let mut handle = stdin.lock();

    loop {
        let c = editor_read_key(&mut handle)?;

        if c == ctrl_key('q') {
            break;
        }

        if c.is_ascii_control() {
            print!("Control character: {c}\r\n");
        } else {
            print!("{} ('{}')\r\n", c, c as char);
        }
    }

    disable_raw_mode(&stdin)?;

    Ok(())
}
