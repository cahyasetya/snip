use crate::db::Snippet;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn pick(snippets: Vec<Snippet>) -> Option<Snippet> {
    let input = snippets
        .iter()
        .map(|s| s.command.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to launch fzf — is it installed?");

    child.stdin.take().unwrap().write_all(input.as_bytes()).ok();

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None; // user cancelled
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    snippets.into_iter().find(|s| s.command == selected)
}
