use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use sigma_parser::Parser;
use sigma_runtime::Runtime;

#[derive(Default)]
pub struct Shell {
    rt: Runtime,
}

impl Shell {
    pub fn run(self) -> Result<()> {
        println!("Sigma {}", env!("CARGO_PKG_VERSION"));
        let mut rl = DefaultEditor::new()?;
        loop {
            match rl.readline(">>> ") {
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);
                    self.exec(&line);
                }
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => return Ok(()),
                Err(err) => return Err(err.into()),
            }
        }
    }

    fn exec(&self, line: &str) {
        let parser = Parser::new(line);
        for stmt in parser {
            match stmt {
                Ok(stmt) => {
                    println!("{stmt:?}");
                    match self.rt.exec(&stmt) {
                        Ok(Some(x)) => println!("{x}"),
                        Ok(_) => {}
                        Err(err) => eprintln!("{err:?}"),
                    }
                }
                Err(err) => {
                    eprintln!("{err:?}");
                }
            }
        }
    }
}
