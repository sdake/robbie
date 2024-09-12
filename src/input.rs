use anyhow::Result;
use std::io::{self, Write};
use crossterm::{
    event::Event,
    style::Print,
    execute,
    terminal::{enable_raw_mode, disable_raw_mode},
    event::{self, KeyCode, KeyEvent, KeyModifiers, EnableBracketedPaste, DisableBracketedPaste},
};

pub async fn read_user_input() -> Result<String> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;

    execute!(
        std::io::stdout(),
        EnableBracketedPaste,
        Print("Prompt: "),
    )?;
    stdout.flush()?;

    let mut buffer = String::new();

    loop {
        stdout.flush().unwrap();
        match event::read()? {

            // Handle various UI key accesses
            // PASTE: A paste buffer is handled specially.
            // CARRIAGE RETURN: Exit string input.
            // SHIFT-CARRIAGE RETURN: Insert a newline in the current turn.
            // BACKSPACE: Remove the last character entered.
            // ANY OTHER KEY: Store key in buffer.
            Event::Key(event) => {
            match event {
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                } => {
                    println!("");
                    buffer.push('\n');
                    break
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                } => {
                    if !buffer.is_empty() {
                        buffer.pop(); // Remove the last character from buffer
                        stdout.write_all(b"\x08 \x08")?;
                    }
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: _,
                    kind: _,
                    state: _,
                } => {
                    print!("{}", c);
                    buffer.push(c);
                }
                _ => todo!{}
                }
            }
            Event::Paste(data) => {
                buffer.push_str(&data);
                print!("{}", data);
            }
            _ => todo!()
        }
    }

    // Exit raw mode
    execute!(
        std::io::stdout(),
        DisableBracketedPaste,
        Print(""),
    )?;
    let _ = stdout.flush();
    let _ = disable_raw_mode();

    Ok(buffer)
}
