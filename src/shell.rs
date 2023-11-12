use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Default)]
pub struct Shell;

impl Shell {
    pub fn run(self) -> Result<()> {
        println!("Sigma {}", env!("CARGO_PKG_VERSION"));
        let mut rl = DefaultEditor::new()?;
        loop {
            match rl.readline(">>> ") {
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);
                    println!("{line}");
                }
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => return Ok(()),
                Err(err) => return Err(err.into()),
            }
        }
    }
}
