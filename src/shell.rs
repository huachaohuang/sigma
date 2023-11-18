use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use sigma_parser::{Parser, Span};
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
                        Err(err) => print_error(line, err.span, err.message),
                    }
                }
                Err(err) => {
                    print_error(line, err.span, err.message);
                    break;
                }
            }
        }
    }
}

fn print_error(src: &str, span: Span, message: impl ToString) {
    use ariadne::{Color, Label, Report, ReportKind, Source};
    let file = "<stdin>";
    Report::build(ReportKind::Error, file, 0)
        .with_label(
            Label::new((file, span))
                .with_color(Color::Red)
                .with_message(message),
        )
        .finish()
        .eprint((file, Source::from(src)))
        .unwrap();
}
