use std::{
    fmt,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    process::{Command, Stdio},
};

use cuelume::RenderedSound;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    #[default]
    Wav,
    Mp3,
    Mp4,
}

impl AudioFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Mp4 => "mp4",
        }
    }

    #[allow(dead_code)]
    pub const fn next(self) -> Self {
        match self {
            Self::Wav => Self::Mp3,
            Self::Mp3 => Self::Mp4,
            Self::Mp4 => Self::Wav,
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Wav => "WAV",
            Self::Mp3 => "MP3",
            Self::Mp4 => "MP4/AAC",
        })
    }
}

impl std::str::FromStr for AudioFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "wav" => Ok(Self::Wav),
            "mp3" => Ok(Self::Mp3),
            "mp4" | "aac" | "m4a" => Ok(Self::Mp4),
            _ => Err(format!("unsupported format: {value}; use wav, mp3, or mp4")),
        }
    }
}

pub fn export(
    rendered: &RenderedSound,
    path: impl AsRef<Path>,
    format: AudioFormat,
) -> Result<(), ExportError> {
    match format {
        AudioFormat::Wav => export_wav(rendered, path.as_ref()),
        AudioFormat::Mp3 | AudioFormat::Mp4 => export_with_ffmpeg(rendered, path.as_ref(), format),
    }
}

fn export_wav(rendered: &RenderedSound, path: &Path) -> Result<(), ExportError> {
    let specification = hound::WavSpec {
        channels: 2,
        sample_rate: rendered.sample_rate(),
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let writer = BufWriter::new(File::create(path)?);
    let mut writer = hound::WavWriter::new(writer, specification)?;
    for frame in rendered.frames() {
        writer.write_sample(frame.left)?;
        writer.write_sample(frame.right)?;
    }
    writer.finalize().map_err(Into::into)
}

fn export_with_ffmpeg(
    rendered: &RenderedSound,
    path: &Path,
    format: AudioFormat,
) -> Result<(), ExportError> {
    let mut command = Command::new("ffmpeg");
    command
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "f32le", "-ar"])
        .arg(rendered.sample_rate().to_string())
        .args(["-ac", "2", "-i", "pipe:0", "-vn"]);
    match format {
        AudioFormat::Mp3 => {
            command.args(["-codec:a", "libmp3lame", "-b:a", "192k"]);
        }
        AudioFormat::Mp4 => {
            command.args(["-codec:a", "aac", "-b:a", "192k", "-movflags", "+faststart"]);
        }
        AudioFormat::Wav => unreachable!("WAV export does not use FFmpeg"),
    }
    let mut child = command
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                ExportError::FfmpegNotFound
            } else {
                ExportError::Io(error)
            }
        })?;
    {
        let mut stdin = child.stdin.take().ok_or(ExportError::FfmpegPipe)?;
        for frame in rendered.frames() {
            stdin.write_all(&frame.left.to_le_bytes())?;
            stdin.write_all(&frame.right.to_le_bytes())?;
        }
    }
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(ExportError::EncoderFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

#[derive(Debug)]
pub enum ExportError {
    Io(io::Error),
    Wav(hound::Error),
    FfmpegNotFound,
    FfmpegPipe,
    EncoderFailed(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "could not write audio: {error}"),
            Self::Wav(error) => write!(formatter, "could not encode WAV: {error}"),
            Self::FfmpegNotFound => formatter
                .write_str("FFmpeg was not found in PATH; install FFmpeg to export MP3 or MP4"),
            Self::FfmpegPipe => formatter.write_str("could not open the FFmpeg input pipe"),
            Self::EncoderFailed(message) if message.is_empty() => {
                formatter.write_str("FFmpeg audio encoding failed")
            }
            Self::EncoderFailed(message) => {
                write!(formatter, "FFmpeg audio encoding failed: {message}")
            }
        }
    }
}

impl std::error::Error for ExportError {}

impl From<io::Error> for ExportError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<hound::Error> for ExportError {
    fn from(error: hound::Error) -> Self {
        Self::Wav(error)
    }
}
