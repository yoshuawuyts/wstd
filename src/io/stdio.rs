use super::{AsyncInputStream, AsyncOutputStream};
use std::cell::LazyCell;
use wasi::cli::terminal_input::TerminalInput;
use wasi::cli::terminal_output::TerminalOutput;

/// Use the program's stdin as an `AsyncInputStream`.
pub struct Stdin {
    stream: AsyncInputStream,
    terminput: LazyCell<Option<TerminalInput>>,
}

/// Get the program's stdin for use as an `AsyncInputStream`.
pub fn stdin() -> Stdin {
    let stream = AsyncInputStream::new(wasi::cli::stdin::get_stdin());
    Stdin {
        stream,
        terminput: LazyCell::new(|| wasi::cli::terminal_stdin::get_terminal_stdin()),
    }
}

impl std::ops::Deref for Stdin {
    type Target = AsyncInputStream;
    fn deref(&self) -> &AsyncInputStream {
        &self.stream
    }
}
impl std::ops::DerefMut for Stdin {
    fn deref_mut(&mut self) -> &mut AsyncInputStream {
        &mut self.stream
    }
}

impl Stdin {
    /// Check if stdin is a terminal.
    pub fn is_terminal(&self) -> bool {
        LazyCell::force(&self.terminput).is_some()
    }
}

/// Use the program's stdout as an `AsyncOutputStream`.
pub struct Stdout {
    stream: AsyncOutputStream,
    termoutput: LazyCell<Option<TerminalOutput>>,
}

/// Get the program's stdout for use as an `AsyncOutputStream`.
pub fn stdout() -> Stdout {
    let stream = AsyncOutputStream::new(wasi::cli::stdout::get_stdout());
    Stdout {
        stream,
        termoutput: LazyCell::new(|| wasi::cli::terminal_stdout::get_terminal_stdout()),
    }
}

impl Stdout {
    /// Check if stdout is a terminal.
    pub fn is_terminal(&self) -> bool {
        LazyCell::force(&self.termoutput).is_some()
    }
}

impl std::ops::Deref for Stdout {
    type Target = AsyncOutputStream;
    fn deref(&self) -> &AsyncOutputStream {
        &self.stream
    }
}
impl std::ops::DerefMut for Stdout {
    fn deref_mut(&mut self) -> &mut AsyncOutputStream {
        &mut self.stream
    }
}

/// Use the program's stdout as an `AsyncOutputStream`.
pub struct Stderr {
    stream: AsyncOutputStream,
    termoutput: LazyCell<Option<TerminalOutput>>,
}

/// Get the program's stdout for use as an `AsyncOutputStream`.
pub fn stderr() -> Stderr {
    let stream = AsyncOutputStream::new(wasi::cli::stderr::get_stderr());
    Stderr {
        stream,
        termoutput: LazyCell::new(|| wasi::cli::terminal_stderr::get_terminal_stderr()),
    }
}

impl Stderr {
    /// Check if stderr is a terminal.
    pub fn is_terminal(&self) -> bool {
        LazyCell::force(&self.termoutput).is_some()
    }
}

impl std::ops::Deref for Stderr {
    type Target = AsyncOutputStream;
    fn deref(&self) -> &AsyncOutputStream {
        &self.stream
    }
}
impl std::ops::DerefMut for Stderr {
    fn deref_mut(&mut self) -> &mut AsyncOutputStream {
        &mut self.stream
    }
}

#[cfg(test)]
mod test {
    use crate::io::AsyncWrite;
    use crate::runtime::block_on;
    #[test]
    // No internal predicate. Run test with --nocapture and inspect output manually.
    fn stdout_println_hello_world() {
        block_on(async {
            let mut stdout = super::stdout();
            let term = if stdout.is_terminal() { "is" } else { "is not" };
            stdout
                .write_all(format!("hello, world! stdout {term} a terminal\n",).as_bytes())
                .await
                .unwrap();
        })
    }
    #[test]
    // No internal predicate. Run test with --nocapture and inspect output manually.
    fn stderr_println_hello_world() {
        block_on(async {
            let mut stdout = super::stdout();
            let term = if stdout.is_terminal() { "is" } else { "is not" };
            stdout
                .write_all(format!("hello, world! stderr {term} a terminal\n",).as_bytes())
                .await
                .unwrap();
        })
    }
}
