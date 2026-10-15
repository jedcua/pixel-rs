use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

/// Render a .pxl pixel art file to a PNG image.
#[derive(Parser)]
#[command(
    version,
    override_usage = "pxl [-g] [-o output] [-s scale] file",
    args_override_self = true
)]
struct Cli {
    /// Path to the .pxl file. Use `-` to read from stdin.
    #[arg(value_name = "file")]
    input: PathBuf,

    /// Path to the output PNG. Use `-` to write to stdout. The default is the
    /// input path with a .png extension, or stdout if the input is stdin.
    #[arg(short, long, value_name = "output")]
    output: Option<PathBuf>,

    /// Size in output pixels of one .pxl char.
    #[arg(short, long, value_name = "scale", default_value_t = NonZeroU32::MIN)]
    scale: NonZeroU32,

    /// Draw a grid between the pixels, with x labels at the top and y labels
    /// at the left. Lines at multiples of 10 are darker. Use a scale of 4 or more.
    #[arg(short, long)]
    grid: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    let input = describe(&cli.input, "stdin");
    let source = read_source(&cli.input).map_err(|err| format!("cannot read {input}: {err}"))?;
    let image = pxl::parse(&source).map_err(|err| format!("{input}: {err}"))?;
    let canvas = if cli.grid {
        pxl::render_grid(&image, cli.scale)
    } else {
        pxl::render(&image, cli.scale)
    }
    .map_err(|err| format!("{input}: {err}"))?;

    let output = match &cli.output {
        Some(path) => path.clone(),
        None if is_stdio(&cli.input) => PathBuf::from("-"),
        None => cli.input.with_extension("png"),
    };
    let writer: Box<dyn Write> = if is_stdio(&output) {
        Box::new(io::stdout().lock())
    } else {
        let file = File::create(&output)
            .map_err(|err| format!("cannot create {}: {err}", output.display()))?;
        Box::new(file)
    };

    let write_error =
        |err: &dyn Display| format!("cannot write {}: {err}", describe(&output, "stdout"));
    let mut writer = BufWriter::new(writer);
    pxl::write_png(&canvas, &mut writer).map_err(|err| write_error(&err))?;
    writer.flush().map_err(|err| write_error(&err))
}

fn read_source(path: &Path) -> io::Result<String> {
    if is_stdio(path) {
        let mut source = String::new();
        io::stdin().lock().read_to_string(&mut source)?;
        Ok(source)
    } else {
        fs::read_to_string(path)
    }
}

/// Returns true if `path` is the `-` operand for stdin or stdout.
fn is_stdio(path: &Path) -> bool {
    path.as_os_str() == "-"
}

/// Returns a name for `path` to use in messages. `stream` is the name for `-`.
fn describe(path: &Path, stream: &str) -> String {
    if is_stdio(path) {
        stream.into()
    } else {
        path.display().to_string()
    }
}
