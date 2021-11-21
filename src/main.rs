use std::cmp::min;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::path::{Path, PathBuf};
use std::{fs, iter};
use structopt::StructOpt;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

/// Format and print the arguments to stderr then exit the program with code 1
#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
        std::process::exit(1);
    };
}

/// Write arguments to buffer
///
/// First argument: buffer to write to
#[macro_export]
macro_rules! output {
    ($buffer:expr, $first:expr) => {
        write!($buffer, "{}", $first).unwrap();
    };
    ($buffer:expr, $first:expr, $($arg:tt)+) => {
        write!($buffer, "{}", $first).unwrap();
        output!($buffer, $($arg)+);
    };
}

const SEPARATOR: &str = "From ";

type Mail = String;

trait FindField {
    fn find_field<'a>(mail: &'a str, field: &'a str) -> Option<&'a str>;
}
impl FindField for Mail {
    fn find_field<'a>(mail: &'a str, field: &'a str) -> Option<&'a str> {
        let res = mail.lines().find(|p| p.starts_with(field));
        match res {
            Some(x) => Some(x.trim()),
            _ => None,
        }
    }
}

type Mails = Vec<Mail>;

trait MailsConstructor {
    type Output;
    fn from_filename<P>(filename: P) -> Self::Output
    where
        P: AsRef<Path>;
}
impl MailsConstructor for Mails {
    type Output = Result<Self, Box<dyn Error>>;
    fn from_filename<P>(filename: P) -> Self::Output
    where
        P: AsRef<Path>,
    {
        let mut mails = Self::new();
        let contents = fs::read_to_string::<P>(filename)?;
        let mut raw_mails = contents.split("\n\n").peekable();

        while let Some(first) = raw_mails.next() {
            {
                let mut mail = first.to_owned();

                // If next element is new mail, break, else add it to mail
                while let Some(&s) = raw_mails.peek() {
                    if s.starts_with(SEPARATOR) {
                        break;
                    } else {
                        raw_mails.next();
                        mail.push_str(s);
                    }
                }
                mails.push(mail);
            }
        }
        Ok(mails)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Simple mail reader",
    about = "A simple mail reader CLI made with Rust to view unix mails."
)]
struct Opt {
    /// Users to skip reading mail from
    #[structopt(short, long)]
    skip: Option<Vec<String>>,

    /// Path to the directory containing mail
    #[structopt(short, long, parse(from_os_str), default_value = "/var/mail")]
    path: PathBuf,

    /// User to read mail from
    #[structopt(name = "USER")]
    user: Option<String>,
}

fn main() {
    const HEADER_HEIGHT: u16 = 2;

    let opt = Opt::from_args();
    let stdin = stdin();
    let (_size_w, size_h) = termion::terminal_size().unwrap();

    let mut mails: Mails;

    if let Some(user) = opt.user {
        println!("Getting mail from user: {}", user);
        let filename = opt.path.join(&user);
        mails = Mails::from_filename(&filename).unwrap_or_else(|_| {
            exit!(
                "Error: User has no mail file in folder: {}",
                filename.display()
            );
        });
    } else {
        println!("Getting mail from all users in folder");
        mails = Mails::new();

        let skip = opt.skip.unwrap_or(Vec::new());
        for mail_file in fs::read_dir(&opt.path)
            .expect("Unable to read mail folder")
            .filter_map(Result::ok)
        {
            let user = mail_file
                .file_name()
                .to_str()
                .unwrap_or_default()
                .to_owned();
            if skip.contains(&user) {
                println!("Skipping user: {}", user);
                continue;
            }
            if let Ok(mail) = Mails::from_filename(&mail_file.path()) {
                mails.extend(mail);
            }
        }
    }

    let total_mails = mails.len();

    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    let mut current_mail: usize = 0;
    let mut current_line: usize = 0;
    let mut delete_started = false;

    // Null key is to display first mail without pressing key
    for c in iter::once(Ok(Key::Null)).chain(stdin.keys()) {
        let chr = c.unwrap();
        match chr {
            Key::Esc | Key::Char('q') => break,
            Key::Char('d') => {
                if delete_started {
                    todo!("Delete current mail");
                } else {
                    delete_started = true;
                }
            }

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
        let mail = mails[current_mail].trim();
        let lines = mail.lines();
        let total_lines = lines.clone().count();
        match chr {
            Key::Up => current_line = min(current_line.saturating_sub(1), total_lines - 1),
            Key::Down => current_line = min(current_line.saturating_add(1), total_lines - 1),
            _ => {}
        }

        output!(screen, termion::clear::All, cursor::Goto(1, 1));

        let to = Mail::find_field(mail, "To: ").unwrap_or("Unknown");
        let date = Mail::find_field(mail, "Date: ")
            .unwrap_or("Unknown")
            .split_whitespace()
            .take(6) // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toString#description
            .collect::<Vec<&str>>()
            .join(" ");
        let instructions =
            "PgUp/PgDown/Home/End=prev/next/first/last  ↑/↓=prev/next line  q/esc=quit";
        output!(
            screen,
            termion::style::Underline,
            cursor::Goto(1, 1),
            format!(
                "Reading mail {}/{}\t{}",
                current_mail + 1,
                total_mails,
                instructions
            ),
            cursor::Goto(1, 2),
            format!("{}\t{}", to, date),
            termion::style::NoUnderline
        );

        output!(screen, cursor::Goto(1, HEADER_HEIGHT + 1));
        for line in lines
            .skip(current_line)
            .take(usize::from(size_h) - HEADER_HEIGHT as usize)
        {
            write!(screen, "{}\r\n", line).unwrap();
        }

        screen.flush().unwrap();
    }

    output!(screen, cursor::Show, cursor::Restore);
}
