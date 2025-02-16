use std::fmt;
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use clap::{Arg, ArgMatches, ColorChoice, ValueHint};
use clap::parser::ValuesRef;
use image::{ImageError, ImageFormat, ImageReader};
use image::error::{UnsupportedError, UnsupportedErrorKind};
use glob::glob;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug)]
enum ImcoError {
    // IO Errors; Reason, Path
    FailedFileRead(String, String),
    FailedFileWrite(String, String),
    InvalidBatching,
    // Format
    InvalidFormat(String),
    NoDestFormat,
    // file path, [hint]
    Decoding(String, String),
    Encoding(String, String),
    Unsupported(String, String),
    InternalConversionError(String),
    ResourceLimitReached(String),
    // Error, Pattern
    BatchPattern(String, String),
    BatchReadEntry(String)
}

impl fmt::Display for ImcoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImcoError::FailedFileRead(cause, path) => {write!(f, "Failed reading '{path}' => {cause}")}
            ImcoError::FailedFileWrite(cause, path) => {write!(f, "Failed writing '{path}' => {cause}")}
            ImcoError::InvalidFormat(fmt) => {write!(f, "Unknown format {fmt}, use --help for a list")}
            ImcoError::InvalidBatching => {write!(f, "Batching is only allowed when specifying an output format (using --output-format)")}
            ImcoError::NoDestFormat => {write!(f, "No output format provided (use --output-format)")}
            ImcoError::Decoding(path, hint) => {write!(f, "Error during decoding of '{path}' => {hint}")}
            ImcoError::Encoding(path, hint) => {write!(f, "Error during encoding of '{path}' => {hint}")}
            ImcoError::Unsupported(path, hint) => {write!(f, "{hint} during conversion of '{path}'")}
            ImcoError::InternalConversionError(path) => {write!(f, "Internal error during conversion of '{path}'")}
            ImcoError::ResourceLimitReached(path) => {write!(f, "Exceeded resource limitation during conversion of '{path}'")},
            ImcoError::BatchPattern(err, pat) => { write!(f, "Failed to collect files using glob, '{pat}' => {err}") }
            ImcoError::BatchReadEntry(err) => { write!(f, "Failed to read directory entry using glob => {err}") },
        }
    }
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
    ImageFormat::from_extension(f).ok_or(ImcoError::InvalidFormat(f.to_owned()))
}

fn mk_format_fp(f: &String) -> ImcoResult<ImageFormat> {
    ImageFormat::from_extension(std::path::Path::new(f).extension().ok_or(ImcoError::InvalidFormat(f.to_owned()))?).ok_or(ImcoError::InvalidFormat(f.to_owned()))
}

fn mk_unsupported_str(u: UnsupportedError) -> String {
    match u.kind() {
        UnsupportedErrorKind::Color(c) => {
            format!("Unsupported color ({:?})", c)
        }
        UnsupportedErrorKind::Format(f) => {
            format!("Unsupported or not allowed image format ({})", f)
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
            ImageError::Parameter(_) => { ImcoError::InternalConversionError(img_path) }
            ImageError::Limits(_) => { ImcoError::ResourceLimitReached(img_path) }
            ImageError::Unsupported(u) => {ImcoError::Unsupported(img_path, mk_unsupported_str(u))}
            ImageError::IoError(e) => { io_error_convert::<String>(Err(e), &*img_path, false).unwrap_err() }
        }
    })
}

fn mk_filename(p: &String, fmt: ImageFormat) -> String {
    std::path::Path::new(&p).file_stem()
        .and_then(|t| {Some(format!("{}.{}", t.to_str().unwrap(), fmt.extensions_str()[0]))})
        .or(Some(p.to_string() + fmt.extensions_str()[0]))
        .unwrap()
}

fn join_path(p: &String, fmt: ImageFormat, stem: &String) -> String {
    std::path::Path::new(stem).join(mk_filename(p, fmt)).to_str().unwrap().to_string()
}

fn individual_process(path: String, output: Option<String>, i_fmt: Option<ImageFormat>, o_fmt: Option<ImageFormat>, batch: bool) -> ImcoResult<(String, Option<ImageFormat>, ImageFormat)> {
    if output.is_none() && o_fmt.is_none() { return Err(ImcoError::NoDestFormat) }
    
    let mut raw_image = imread(&*path)?;
    let org_fmt = if i_fmt.is_some() {
        raw_image.set_format(i_fmt.unwrap());
        i_fmt
    } else { raw_image.format() };
    let image = image_err_convert(raw_image.decode(), path.clone())?;
    
    Ok(if o_fmt.is_some() {
        let fmt = o_fmt.unwrap();
        let output = if batch { join_path(&path, fmt, &output.unwrap()) } else { if output.is_some() { output.unwrap() } else { mk_filename(&path, fmt) } };
        image_err_convert(image.save_with_format(&output, fmt), path)?;
        (output, org_fmt, fmt)
    } else {
        if batch { return Err(ImcoError::InvalidBatching) }
        let output = output.unwrap();
        let aif = mk_format_fp(&output)?;
        image_err_convert(image.save(&output), path)?;
        (output, org_fmt, aif)
    })
}

fn process(couples: Vec<(&String, Option<&&String>)>, i_fmt_s: Option<&String>, o_fmt_s: Option<&String>, batch: bool) -> ImcoResult<()> {
    let i_fmt = if i_fmt_s.is_some() { Some(mk_format(i_fmt_s.unwrap())?) } else {None};
    let o_fmt = if o_fmt_s.is_some() { Some(mk_format(o_fmt_s.unwrap())?) } else {None};

    for couple in couples {
        let res = individual_process(couple.0.to_string(), couple.1.and_then(|t| { Some(t.to_string()) }), i_fmt, o_fmt, batch)?;
        if res.1.is_some() {
            println!("{} ({}) -> {} ({})", couple.0, res.1.unwrap().extensions_str()[0], res.0, res.2.extensions_str()[0])
        } else {
            println!("{} -> {} ({})", couple.0, res.0, res.2.extensions_str()[0])
        }
    }
    
    Ok(())
}

fn expand_patterns_to_files(patterns: ValuesRef<String>) -> ImcoResult<Vec<String>> {
    let mut files = Vec::new();
    for pattern in patterns {
        match glob(&pattern) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => files.push(path.to_str().unwrap().to_string()),
                        Err(e) => return Err(ImcoError::BatchReadEntry(e.to_string())),
                    }
                }
            }
            Err(e) => return Err(ImcoError::BatchPattern(e.to_string(), pattern.to_string())),
        }
    }
    Ok(files)
}

fn parse_and_execute(matches: ArgMatches) -> Result<(), ImcoError> {
    let batch = matches.get_flag("batch");

    let mut couples = vec![];

    let input_files: Vec<String> = if batch {
        expand_patterns_to_files(matches.get_many::<String>("input").unwrap())?
    } else {
        matches.get_many::<String>("input").unwrap().map(|x| {x.to_string()}).collect()
    };

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

fn main() {
    let matches = clap::Command::new(NAME)
        .about(DESCRIPTION)
        .version(VERSION)
        .color(ColorChoice::Never)
        .disable_version_flag(true)
        .after_help("Accepted file formats:\n avif, jpg / jpeg / jfif, png / apng,\n gif, webp, tif / tiff, tga, dds,\n bmp, ico, hdr, exr, pbm / pam / ppm / pgm,\n ff, qoi, pcx")
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
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .help("Print version")
            .action(clap::ArgAction::Version))
        .get_matches();
    
    let res = parse_and_execute(matches);
    if res.is_err() {
        println!("{}", res.unwrap_err())
    }
}
