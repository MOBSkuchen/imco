use std::fs::File;
use std::io::{BufReader, Cursor, ErrorKind};
use image::{ImageFormat, ImageReader};

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
    io_error_convert(ImageReader::open(path), path, true)
}

fn mk_format(f: &String) -> ImcoResult<ImageFormat> {
    ImageFormat::from_extension(f).ok_or(ImcoError::InvalidFormat(f.to_owned()))
}

fn main() -> Result<(), ImcoError> {

}
