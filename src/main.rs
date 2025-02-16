use std::fs::File;
use std::io::{BufReader, ErrorKind};
use clap::{Arg, ColorChoice, ValueHint};
use image::{ImageError, ImageFormat, ImageReader};
use image::error::{UnsupportedError, UnsupportedErrorKind};
use image::flat::Error;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug)]
enum ImcoError {
    // IO Errors; Reason, Path
    FailedFileRead(String, String),
    FailedFileWrite(String, String),
    
    // Format
    InvalidFormat(String),
    
    NoDestFormat,
    
    // file path, [hint]
    Decoding(String, String),
    Encoding(String, String),
    InternalConversionError(String),
    ResourceLimitReached(String),
    Unsupported(String, String)
}

type ImcoResult<T> = Result<T, ImcoError>;
type ImReader = ImageReader<BufReader<File>>;

fn io_error_convert<T>(res: Result<T, std::io::Error>, file_path: &str, is_read: bool) -> Result<T, ImcoError> {
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
    io_error_convert::<ImReader>(ImageReader::open(path), path, true)
}

fn mk_format(f: &String) -> ImcoResult<ImageFormat> {
    ImageFormat::from_extension("a.".to_string() + f).ok_or(ImcoError::InvalidFormat(f.to_owned()))
}

fn mk_unsupported_str(u: UnsupportedError) -> String {
    match u.kind() {
        UnsupportedErrorKind::Color(c) => {
            "Unsupported color during conversion".to_string()
        }
        UnsupportedErrorKind::Format(f) => {
            "Unsupported image format or not allowed format conversion".to_string()
        }
        UnsupportedErrorKind::GenericFeature(gf) => {
            gf
        },
        _ => "Other".to_string(),
    }
}

fn image_err_convert<T>(res: Result<T, ImageError>, img_path: String) -> Result<T, ImcoError> {
    res.map_err(|e| {
        match e {
            ImageError::Decoding(de) => { ImcoError::Decoding(img_path, de.to_string()) }
            ImageError::Encoding(ee) => { ImcoError::Encoding(img_path, ee.to_string()) }
            ImageError::Parameter(p) => { ImcoError::InternalConversionError(img_path) }
            ImageError::Limits(l) => { ImcoError::ResourceLimitReached(img_path) }
            ImageError::Unsupported(u) => {ImcoError::Unsupported(img_path, mk_unsupported_str(u))}
            ImageError::IoError(e) => { io_error_convert::<String>(Err(e), &*img_path, false).unwrap_err() }
        }
    })
}

fn individual_process(path: String, output: Option<String>, i_fmt: Option<ImageFormat>, o_fmt: Option<ImageFormat>) -> ImcoResult<String> {
    if output.is_none() && o_fmt.is_none() { return Err(ImcoError::NoDestFormat) }
    
    let mut raw_image = imread(&*path)?;
    if i_fmt.is_some() { raw_image.set_format(i_fmt.unwrap()) }
    let image = image_err_convert(raw_image.decode(), path.clone())?;
    
    Ok(if o_fmt.is_some() {
        let fmt = o_fmt.unwrap();
        // TODO : Better auto output
        let output = if output.is_some() { output.unwrap() } else { path.clone() + fmt.extensions_str()[0] };
        image_err_convert(image.save_with_format(&output, fmt), path)?;
        output
    } else {
        let output = output.unwrap();
        image_err_convert(image.save(&output), path)?;
        output
    })
}

fn process(couples: Vec<(&&String, Option<&&String>)>, i_fmt_s: Option<&String>, o_fmt_s: Option<&String>, batch: bool) -> ImcoResult<()> {
    let i_fmt = if i_fmt_s.is_some() { Some(mk_format(i_fmt_s.unwrap())?) } else {None};
    let o_fmt = if o_fmt_s.is_some() { Some(mk_format(o_fmt_s.unwrap())?) } else {None};

    for couple in couples {
        individual_process(couple.0.to_string(), couple.1.and_then(|t| { Some(t.to_string()) }), i_fmt, o_fmt)?;
    }
    
    Ok(())
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
    
    // TODO: Add batch processing
    let batch = matches.get_flag("batch");

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
    
    let i_fmt = matches.get_one::<String>("input-format");
    let o_fmt = matches.get_one::<String>("output-format");
    
    process(couples, i_fmt, o_fmt, batch)
}
