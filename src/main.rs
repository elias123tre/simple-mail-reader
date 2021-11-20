use std::cmp::min;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

const SEPARATOR: &str = "\n\nFrom ";
const MAIL_FOLDER: &str = "/var/mail";

type Mail = String;

type Mails = Vec<Mail>;

trait MailsExtend {
    type Output;
    fn from_filename<P>(filename: P) -> Self::Output
    where
        P: AsRef<Path>;
}
impl MailsExtend for Mails {
    type Output = Result<Self, Box<dyn Error>>;
    fn from_filename<P>(filename: P) -> Self::Output
    where
        P: AsRef<Path>,
    {
        let mut mails = Self::new();
        let contents = fs::read_to_string::<P>(filename)?;

        let mut rest = &contents[..];
        while let Some(split_i) = rest.find(SEPARATOR) {
            mails.push(rest[..split_i].to_owned());
            rest = &rest[split_i..];
        }
        Ok(mails)
    }
}

fn main() {
    let stdin = stdin();
    let (_size_w, size_h) = termion::terminal_size().unwrap();

    // TODO: Parse arguments (https://docs.rs/structopt/0.3.25/structopt/)
    // TODO: Read mail from all users

    let mails: Mails;

    // If user argument is provided
    if let Some(user) = env::args().skip(1).next() {
        let filename = Path::new(MAIL_FOLDER).join(&user);
        mails = Mails::from_filename(filename).expect("User has no mail file");
    } else {
        // Else: read all mails
        for dir in fs::read_dir(MAIL_FOLDER)
            .expect("Unable to read mail folder")
            .filter_map(Result::ok)
        {
            println!("{:?}", dir.file_name());
        }
        mails = Mails::new();
    }

    std::process::exit(0);

    // let read_mail = |user: String| ;

    // let mails = read_mail("");
    let total_mails = mails.len();

    println!("Press any key to read first mail. Press 'q' or ESC anytime to quit, use Page Up and Page Down to scroll between mails. Scroll lines with arrow keys.");

    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    let mut current_mail: usize = 0;
    let mut current_line: usize = 0;

    for c in stdin.keys() {
        let chr = c.unwrap();
        match chr {
            Key::Esc | Key::Char('q') => break,
            Key::Left => println!("←"),
            Key::Right => println!("→"),
            Key::Backspace => println!("×"),
            Key::Char('d') => println!("Deleting mail"),

            Key::PageUp => {
                current_mail = min(current_mail.saturating_sub(1), total_mails - 1);
                current_line = 0
            }
            Key::PageDown => {
                current_mail = min(current_mail.saturating_add(1), total_mails - 1);
                current_line = 0
            }
            Key::Home => {
                current_mail = 0;
                current_line = 0;
            }
            Key::End => {
                current_mail = total_mails - 1;
                current_line = 0;
            }
            _ => {}
        }
        let mail = &mails[current_mail].trim();
        let lines = mail.lines();
        let total_lines = lines.clone().count();
        match chr {
            Key::Up => current_line = min(current_line.saturating_sub(1), total_lines - 1),
            Key::Down => current_line = min(current_line.saturating_add(1), total_lines - 1),
            _ => {}
        }
        write!(
            screen,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 2)
        )
        .unwrap();
        for line in lines.skip(current_line).take(usize::from(size_h) - 2) {
            write!(screen, "{}\r\n", line).unwrap();
        }
        write!(
            screen,
            "{}{}{}{}{}{}",
            termion::cursor::Goto(1, 1),
            termion::style::Underline,
            format!("Reading mail {}/{}    ", current_mail + 1, total_mails),
            mail.lines()
                .find(|p| p.starts_with("Date: "))
                .unwrap_or("Unknown")
                .trim()
                .split_whitespace()
                .take(6) // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toString#description
                .collect::<Vec<&str>>()
                .join(" "),
            "    PgUp/Down=prev/next mail  ↑/↓=prev/next line  q/esc=quit",
            termion::style::NoUnderline
        )
        .unwrap();
        screen.flush().unwrap();
    }

    write!(
        screen,
        "{}{}",
        termion::cursor::Show,
        termion::cursor::Restore
    )
    .unwrap();
}
