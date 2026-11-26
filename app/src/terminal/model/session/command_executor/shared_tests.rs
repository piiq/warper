use super::*;

#[test]
#[cfg(feature = "local_tty")]
fn environment_values_preserve_shell_metacharacters() {
    let pairs = [("PATH", "/bin:/dir with spaces"), ("VALUE", "a'b;$HOME")];
    for shell_type in [ShellType::Bash, ShellType::Zsh] {
        assert_eq!(
            serialize_environment_variables(pairs, shell_type),
            "PATH='/bin:/dir with spaces' VALUE='a'\"'\"'b;$HOME'"
        );
    }
    assert_eq!(
        serialize_environment_variables(pairs, ShellType::Fish),
        "set -x PATH '/bin:/dir with spaces'; set -x VALUE 'a\\'b;$HOME';"
    );
    assert_eq!(
        serialize_environment_variables(pairs, ShellType::PowerShell),
        "$env:PATH = '/bin:/dir with spaces'; $env:VALUE = 'a''b;$HOME';"
    );
}
