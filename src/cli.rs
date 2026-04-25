use crate::verdict::Verdict;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(args: impl IntoIterator<Item = String>) -> CliResponse {
    let mut args = args.into_iter();
    let _program = args.next();

    match args.next().as_deref() {
        None => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: String::new(),
        },
        Some("--help") | Some("-h") => CliResponse {
            exit_code: 0,
            stdout: help_text(),
            stderr: String::new(),
        },
        Some("--version") | Some("-V") => CliResponse {
            exit_code: 0,
            stdout: format!("lfg {}\n", env!("CARGO_PKG_VERSION")),
            stderr: String::new(),
        },
        Some(argument) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: unknown argument: {argument}\n"),
        },
    }
}

fn help_text() -> String {
    "\
lfg is a local pre-install guard for package managers.

Usage: lfg [OPTIONS]

Options:
  -h, --help       Print help
  -V, --version    Print version
"
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_returns_ask_without_output() {
        let response = run(["lfg".to_owned()]);

        assert_eq!(response.exit_code, 20);
        assert!(response.stdout.is_empty());
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn help_returns_success_with_usage() {
        let response = run(["lfg".to_owned(), "--help".to_owned()]);

        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("Usage: lfg"));
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn unknown_arg_returns_cli_misuse() {
        let response = run(["lfg".to_owned(), "--bad".to_owned()]);

        assert_eq!(response.exit_code, 1);
        assert!(response.stdout.is_empty());
        assert_eq!(response.stderr, "lfg: unknown argument: --bad\n");
    }
}
