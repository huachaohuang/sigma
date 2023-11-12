use anyhow::Result;

mod shell;
use shell::Shell;

fn main() -> Result<()> {
    Shell::default().run()
}
