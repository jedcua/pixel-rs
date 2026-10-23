use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser};

mod editor;

/// Render a .pxl pixel art file to a PNG image or to the terminal, edit it
/// in the terminal, set pixels in it, or convert a PNG or JPG image to a
/// .pxl file.
#[derive(Parser)]
#[command(
    version,
    override_usage = "pxl [-g] [-o output] [-s scale] file\n       pxl -c [-o output] [-s scale] image\n       pxl -p file\n       pxl -e file\n       pxl --set [-o output] file edit...",
    args_override_self = true
)]
struct Cli {
    /// Path to the .pxl file, or to the PNG or JPG image with -c. Use `-` to
    /// read from stdin.
    #[arg(value_name = "file")]
    input: PathBuf,

    /// Pixel edits for --set, each in the form `x,y=K`.
    #[arg(value_name = "edit", requires = "set")]
    edits: Vec<pxl::PixelEdit>,

    /// Path to the output PNG, or to the output .pxl file with -c and --set.
    /// Use `-` to write to stdout. The default is the input path with a .png
    /// extension (.pxl with -c, the input path with --set), or stdout if the
    /// input is stdin.
    #[arg(short, long, value_name = "output")]
    output: Option<PathBuf>,

    /// Size in output pixels of one .pxl char. With -c, size in image pixels
    /// of one .pxl char.
    #[arg(short, long, value_name = "scale", default_value_t = NonZeroU32::MIN)]
    scale: NonZeroU32,

    /// Draw a grid between the pixels, with x labels at the top and y labels
    /// at the left. Lines at multiples of 10 are darker. Use a scale of 4 or more.
    #[arg(short, long)]
    grid: bool,

    #[command(flatten)]
    mode: Mode,
}

/// The modes other than render. A command uses one mode at most.
#[derive(Args)]
#[group(multiple = false)]
// Each bool is one flag, and the group lets at most one be true. An enum
// would need a flag syntax that clap derive does not have for these flags.
#[allow(clippy::struct_excessive_bools)]
struct Mode {
    /// Convert a PNG or JPG image of pixel art to a .pxl file. Each block of
    /// scale x scale image pixels becomes one .pxl char.
    #[arg(short, long, conflicts_with = "grid")]
    convert: bool,

    /// Open the file in the terminal editor, and do not render a PNG.
    #[arg(short, long, conflicts_with_all = ["output", "scale", "grid"])]
    edit: bool,

    /// Print the image to stdout with 24-bit terminal colors, and do not
    /// render a PNG. Each char shows two pixels, one above the other.
    #[arg(short, long, conflicts_with_all = ["output", "scale", "grid"])]
    print: bool,

    /// Set pixels and write the .pxl file. Each edit is `x,y=K`: `x` and `y`
    /// are a 0-based coordinate or an inclusive range `a-b`, and `K` is a
    /// palette key. A later edit wins where edits overlap. If an edit is not
    /// valid, the file does not change.
    #[arg(long, requires = "edits", conflicts_with_all = ["scale", "grid"])]
    set: bool,
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
    if cli.mode.edit {
        if is_stdio(&cli.input) {
            return Err("cannot edit stdin; give a file path".into());
        }
        return editor::run(&cli.input);
    }
    let input = describe(&cli.input, "stdin");
    let bytes = read_input(&cli.input).map_err(|err| format!("cannot read {input}: {err}"))?;
    if cli.mode.convert {
        let document = pxl::convert(&bytes, cli.scale).map_err(|err| format!("{input}: {err}"))?;
        return write_output(&output_path(cli, "pxl"), |writer| {
            writer.write_all(document.to_pxl().as_bytes())
        });
    }
    let source = String::from_utf8(bytes).map_err(|err| format!("cannot read {input}: {err}"))?;
    if cli.mode.set {
        return set_pixels(cli, &input, &source);
    }
    let image = pxl::parse(&source).map_err(|err| format!("{input}: {err}"))?;
    if cli.mode.print {
        return print_ansi(&image);
    }
    let canvas = if cli.grid {
        pxl::render_grid(&image, cli.scale)
    } else {
        pxl::render(&image, cli.scale)
    }
    .map_err(|err| format!("{input}: {err}"))?;
    write_output(&output_path(cli, "png"), |writer| {
        pxl::write_png(&canvas, writer)
    })
}

/// Applies the `--set` edits to the `.pxl` `source`, writes the result, and
/// prints the number of changed pixels to stderr.
fn set_pixels(cli: &Cli, input: &str, source: &str) -> Result<(), String> {
    let mut document = pxl::Document::parse(source).map_err(|err| format!("{input}: {err}"))?;
    let changed = document
        .apply_edits(&cli.edits)
        .map_err(|err| format!("{input}: {err}"))?;
    let output = cli.output.as_ref().unwrap_or(&cli.input);
    write_output(output, |writer| {
        writer.write_all(document.to_pxl().as_bytes())
    })?;
    let noun = if changed == 1 { "pixel" } else { "pixels" };
    eprintln!("changed {changed} {noun}");
    Ok(())
}

/// Returns the `-o` path, or else stdout for stdin input, or else the input
/// path with `extension`.
fn output_path(cli: &Cli, extension: &str) -> PathBuf {
    match &cli.output {
        Some(path) => path.clone(),
        None if is_stdio(&cli.input) => PathBuf::from("-"),
        None => cli.input.with_extension(extension),
    }
}

/// Calls `write` with a buffered writer for `output`, or for stdout if
/// `output` is `-`.
fn write_output<E: Display>(
    output: &Path,
    write: impl FnOnce(&mut BufWriter<Box<dyn Write>>) -> Result<(), E>,
) -> Result<(), String> {
    let sink: Box<dyn Write> = if is_stdio(output) {
        Box::new(io::stdout().lock())
    } else {
        let file = File::create(output)
            .map_err(|err| format!("cannot create {}: {err}", output.display()))?;
        Box::new(file)
    };

    let write_error =
        |err: &dyn Display| format!("cannot write {}: {err}", describe(output, "stdout"));
    let mut writer = BufWriter::new(sink);
    write(&mut writer).map_err(|err| write_error(&err))?;
    writer.flush().map_err(|err| write_error(&err))
}

fn read_input(path: &Path) -> io::Result<Vec<u8>> {
    if is_stdio(path) {
        let mut bytes = Vec::new();
        io::stdin().lock().read_to_end(&mut bytes)?;
        Ok(bytes)
    } else {
        fs::read(path)
    }
}

/// Writes `image` to stdout as terminal text.
fn print_ansi(image: &pxl::Image) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(pxl::render_ansi(image).as_bytes())
        .and_then(|()| stdout.flush())
        .map_err(|err| format!("cannot write stdout: {err}"))
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
