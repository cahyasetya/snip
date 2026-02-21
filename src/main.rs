mod db;
mod picker;

use std::process::Command;

fn run_command(command: &str) -> anyhow::Result<bool> {
  let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

  let status = Command::new(&shell)
  .arg("-i")
  .arg("-c")
  .arg(command)
  .status()?;

  Ok(status.success())
}

fn main() -> anyhow::Result<()> {
  let conn = db::init()?;
  let args: Vec<String> = std::env::args().collect();

  if args.len() > 1 {
      let command = args[1..].join(" ");
      db::save(&conn, &command)?;
      run_command(&command)?;
  } else {
      // no argument → open fzf picker
      let snippets = db::list(&conn)?;
      if snippets.is_empty() {
          println!("no snippets yet. run: snip \"your command\"");
          return Ok(());
      }

      match picker::pick(&conn, snippets)? {
          Some(selected) => {
              let success = run_command(&selected.command)?;
              if !success {
                  println!("command exited with non-zero status");
              }
          }
          None => {}
      }
  }

  Ok(())
}
