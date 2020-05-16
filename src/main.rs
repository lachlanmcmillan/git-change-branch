mod util;

use std::io;
use std::process::Command;
use crate::util::Events;
use tui::{
    backend::TermionBackend,
    widgets::{Text, List, ListState, Block},
    style::{Style, Color},
    Terminal
};
use termion::{
    raw::IntoRawMode,
    event::Key,
    screen::AlternateScreen
};

enum Commands {
    Checkout,
    Exit
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let branches_string: String = git_read_branches();
    let current_branch: String = git_read_current_branch();

    let branches: Vec<&str> = branches_string
        .lines()
        .map(|x| x.trim())
        .collect(); 

    assert!(!branches.is_empty(), "error: couldn't read branches");
    assert!(!current_branch.is_empty(), "error: couldn't read current branch");

    // find the current branch
    let initial_selected_index = branches.iter().position(|x| *x == current_branch);
    assert!(!initial_selected_index.is_none(), "error: couldn't find current branch in branch list");

    let mut list_state = ListState::default();
    list_state.select(initial_selected_index);

    let command: Commands;
    let events = Events::new();

    {
        // set up termion and tui
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        loop {
            terminal.draw(|mut frame| {
                let size = frame.size();

                let text_items = branches.iter().map(|x| Text::raw(*x));

                let list = List::new(text_items)
                    .block(Block::default())
                    .highlight_style(Style::default().fg(Color::Green))
                    .highlight_symbol("* ");

                frame.render_stateful_widget(list, size, &mut list_state);
            })?;

            match events.next()? {
                Key::Down => {
                    select_next(branches.len(), &mut list_state);
                }
                Key::Up => {
                    select_prev(branches.len(), &mut list_state); 
                }
                Key::Esc | Key::Ctrl('c') | Key::Char('q') => {
                    command = Commands::Exit;
                    break;
                }
                Key::Char('\n') | Key::Char('\r') => {
                    // attempt checkout
                    command = Commands::Checkout;
                    break;
                }
                _ => {}
            }
        }
    }
    // as we exit the above block we return to normal terminal and release 
    // the alternate screen buffer

    match command {
        Commands::Checkout => {
            // get the selected item 
            match list_state.selected() {
                Some(x) => {
                    let branch_name = &branches[x];
                    // println!("{}", branch_name); 
                    let output = git_checkout(&branch_name);
                    println!("{}", output.trim_end()); 
                }
                None => {
                    panic!("error: checkout called without a selected branch")
                }
            }
        }
        _ => { }
    }

    Ok(())
}

pub fn select_next(items_len: usize, list_state: &mut ListState) {
    let i = match list_state.selected() {
        Some(i) => {
            if i >= items_len -1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    list_state.select(Some(i));
}

pub fn select_prev(items_len: usize, list_state: &mut ListState) {
    let i = match list_state.selected() {
        Some(i) => {
            if i == 0 {
                items_len - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    list_state.select(Some(i));
}

fn git_read_current_branch() -> String {
    // git rev-parse --abbrev-ref HEAD
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("failed to call git executable");

    if output.status.success() {
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    } else {
        let stderr = String::from_utf8(output.stderr).unwrap();
        println!("{}", stderr);
        panic!("fail")
    }
}

fn git_read_branches() -> String {
    // git for-each-ref --count=30 --sort=-committerdate refs/heads/ --format='%(refname:short)'
    let output = Command::new("git")
        .args(&["for-each-ref", "--count=20", "--format=%(refname:short)"])
        .output()
        .expect("failed to call git executable");

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .to_string()
}

pub fn git_checkout(branch_name: &str) -> String {
    // git checkout <branch>
    let output = Command::new("git")
        .args(&["checkout", branch_name])
        .output()
        .expect("failed to call git executable");

    // if you try to `git checkout` the current branch it will return success, but 
    // actually print to stderr, so we combine the outputs here to print them for 
    // the user
    let output_vec = output.stdout.into_iter().chain(output.stderr.into_iter()).collect();
    String::from_utf8(output_vec).unwrap()
}