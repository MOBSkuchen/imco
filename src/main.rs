use std::fs::File;
use std::io::{BufReader, ErrorKind};
use clap::{Arg, ColorChoice, ValueHint};
use image::{ImageFormat, ImageReader};

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug)]
enum ImcoError {
    // IO Errors; Reason, Path
    FailedFileRead(String, String),
    FailedFileWrite(String, String),

    InvalidFormat(String)
}

type ImcoResult<T> = Result<T, ImcoError>;
type ImReader = ImageReader<BufReader<File>>;

fn io_error_convert<T, E>(res: Result<T, std::io::Error>, file_path: &str, is_read: bool) -> Result<T, ImcoError> {
    res.map_err(|x| {
        let reason = match x.kind() {
            ErrorKind::NotFound => {"Not found"}
            ErrorKind::PermissionDenied => {"Permission denied"}
            ErrorKind::AlreadyExists => {"Already exists"}
            ErrorKind::NotADirectory => {"Is not a directory"}
            ErrorKind::IsADirectory => {"Is a directory"}
            ErrorKind::StorageFull => {"Storage is full"}
            ErrorKind::FileTooLarge => {"File is too large"}
            _ => {"Unknown (unhandled)"}
        }.to_string();
        if is_read {
            ImcoError::FailedFileRead(reason, file_path.to_string())
        } else {
            ImcoError::FailedFileWrite(reason, file_path.to_string())
        }
    })
}

fn imread(path: &str) -> ImcoResult<ImReader> {
    io_error_convert::<ImReader, ImcoError>(ImageReader::open(path), path, true)
}

fn mk_format(f: &String) -> ImcoResult<ImageFormat> {
    ImageFormat::from_extension(f).ok_or(ImcoError::InvalidFormat(f.to_owned()))
}

fn main() -> Result<(), ImcoError> {
    let matches = clap::Command::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .color(ColorChoice::Never)
        .disable_version_flag(true)
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .help("Displays the version")
            .action(clap::ArgAction::Version))
        .arg(Arg::new("input")
            .help("Input files (seperated by ',')")
            .short('i')
            .long("input")
            .required(true)
            .value_delimiter(',')
            .value_hint(ValueHint::AnyPath)
            .value_name("FILE")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("output")
            .help("Output files (seperated by ',')")
            .short('o')
            .long("output")
            .required(true)
            .value_delimiter(',')
            .value_hint(ValueHint::AnyPath)
            .value_name("FILE")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("input-format")
            .help("Input files formats (see below)")
            .short('f')
            .long("input-format")
            .value_name("FORMAT")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("output-format")
            .help("Output files formats (see below)")
            .short('d')
            .long("output-format")
            .value_name("FORMAT")
            .action(clap::ArgAction::Set))
        .arg(Arg::new("batch")
            .help("Enables batch processing (using patterns to specify multiple files at once)")
            .short('b')
            .long("batch")
            .action(clap::ArgAction::SetTrue))
        .get_matches();
    
    let mut couples = vec![];

    let input_files: Vec<&String> = matches
        .get_many::<String>("input")
        .unwrap()
        .collect();

    let output_files: Vec<&String> = matches
        .get_many::<String>("output")
        .map(|values| values.collect())
        .unwrap_or_default();

    for (i, input_file) in input_files.iter().enumerate() {
        let partner = if output_files.is_empty() {
            None
        } else if i >= output_files.len() {
            Some(output_files.last().unwrap())
        } else {
            Some(&output_files[i])
        };
        
        couples.push((input_file, partner))
    }

    Ok(())
}
