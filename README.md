# pixel-rs

`pixel-rs` is a parser, PNG renderer, terminal editor, pixel setter, and image
converter for `.pxl` files. A `.pxl` file is a plain text file that describes one pixel art
image.

## The `.pxl` format

```pxl
O - #000000FF - outline
X - #FFFFFFFF - fill

XXXXX
XOOOX
XOOOX
XOOOX
XXXXX
```

A `.pxl` file has two sections. One or more blank lines separate them.

1. **Palette.** Each line maps one char to one color, in the form
   `<char> - #RRGGBBAA`.
   - The key is exactly one char. It cannot be whitespace.
   - The spaces around the `-` are optional.
   - The color is `#` and 8 hex digits: red, green, blue, and alpha.
     Uppercase and lowercase digits are both valid.
   - Each key can occur one time only.
   - A label is optional. Add ` - <label>` after the color, for example
     `K - #2B140FFF - outline`. The label can have spaces and dashes.
2. **Pixel data.** Each line is one row of the image. Each char is one pixel.
   - Each char must be a key in the palette.
   - All rows must have the same width.
   - A blank line inside the pixel data is an error.

The parser ignores trailing whitespace on a line. It accepts LF and CRLF line
endings. If the file is not valid, the error gives the line, the column, and
the action to take:

```text
error: art.pxl: line 5, column 2: char `0` is not in the palette; add it to the palette
```

## Setup

This project uses [mise](https://mise.jdx.dev) to install the Rust toolchain
and to run tasks.

```sh
mise trust
mise install
```

To use `pxl` in any directory, install it in `$HOME/.local/bin`:

```sh
mise run install
```

The task builds the release binary and copies it there. Run the task again
after you change the code. If `$HOME/.local/bin` is not in your `PATH`, the
task gives a warning. Then add the directory to `PATH` in your shell profile.

## Usage

```text
pxl [-g] [-o output] [-s scale] file
pxl -c [-o output] [-s scale] image
pxl -p file
pxl -e file
pxl --set [-o output] file edit...
```

Render a file to PNG. Each char becomes one pixel, and the output goes to
`examples/square.png`:

```sh
mise run render -- examples/square.pxl
```

Render at a higher resolution. Each char becomes a 20 x 20 block of pixels,
so the 5 x 5 example becomes a 100 x 100 PNG:

```sh
mise run render -- -s 20 -o square.png examples/square.pxl
```

| Operand or option     | Default                          | Description                                    |
| --------------------- | -------------------------------- | ---------------------------------------------- |
| `file`                | (required)                       | Path to the `.pxl` file, or to the image with `-c`. `-` means stdin. |
| `-o`, `--output`      | Input path with `.png` extension (`.pxl` with `-c`, the input path with `--set`) | Path to the output file. `-` means stdout. |
| `-s`, `--scale`       | `1`                              | Size in output pixels of one char (1 or more). With `-c`, size in image pixels of one char. |
| `-g`, `--grid`        | Off                              | Render the grid view (see below).              |
| `-c`, `--convert`     | Off                              | Convert a PNG or JPG image to a `.pxl` file (see below). You cannot use it with `-g`, `-p`, `-e`, or `--set`. |
| `-p`, `--print`       | Off                              | Print the image in the terminal (see below). You cannot use it with `-o`, `-s`, `-g`, `-c`, `-e`, or `--set`. |
| `-e`, `--edit`        | Off                              | Open the file in the terminal editor (see below). You cannot use it with `-o`, `-s`, `-g`, `-c`, `-p`, `--set`, or `-` as `file`. |
| `--set`               | Off                              | Set pixels by coordinates (see below). You cannot use it with `-s`, `-g`, `-c`, `-p`, or `-e`. |

If `file` is `-` and you do not give `-o`, the output goes to stdout, so you can
use `pxl` in a pipeline:

```sh
generate-art | pxl -s 10 - > art.png
```

### Grid view

Use `-g` to make an image for inspection. The pixel colors stay the same, and
the grid view adds these items:

- A 1 px line between all cells. The line at each multiple of 10 is darker.
- The x coordinate at the top and the y coordinate at the left, every 10 or
  more cells. Coordinates start at 0.
- A checkerboard behind transparent pixels.

Use a scale of 4 or more, so that the cells are larger than the lines:

```sh
mise run render -- -g -s 8 -o grid.png examples/square.pxl
```

### Convert an image

Use `-c` to convert a PNG or JPG image of pixel art to a `.pxl` file. The
output goes to the input path with a `.pxl` extension:

```sh
pxl -c art.png
```

Many images show pixel art at a larger size, where each art pixel is a block
of 2 x 2 or 4 x 4 image pixels. Give the block size with `-s`, so that each
block becomes one char. A 128 x 128 image at scale 4 becomes a 32 x 32 file:

```sh
pxl -c -s 4 -o art.pxl art.png
```

The image width and height must be multiples of the scale. Thus
`pxl -s 4 art.pxl` and `pxl -c -s 4 art.png` reverse each other.

The converter makes the palette as follows:

- Each char gets the most frequent color in its block.
- All fully transparent pixels get the key `.` and the color `#00000000`.
- The other colors get the keys `A`-`Z`, `a`-`z`, `0`-`9`, ASCII
  punctuation, and then other Unicode letters, in order of frequency.
- A PNG keeps its exact colors.
- In a JPG, colors that are almost equal become one color. This removes most
  of the JPG compression noise. A JPG with low quality can still give some
  extra colors that are almost equal. Use the editor to delete them.

### Print in the terminal

Use `-p` to see a file in the terminal, with no PNG and no editor:

```sh
pxl -p examples/basketball.pxl
```

Each char shows two pixels: the upper pixel as the color of `▀`, and the
lower pixel as the background color. Thus a 32 x 32 image takes 32 columns
and 16 lines. A pixel with an alpha below 128 shows the terminal background.
Other pixels show their color and ignore the alpha.

The output uses 24-bit ANSI colors. Use a terminal with 24-bit color, because
other terminals show approximate colors. `-` reads the file from stdin:

```sh
generate-art | pxl -p -
```

### Set pixels

Use `--set` to change pixels by coordinates, without an editor. The file
changes in place:

```sh
pxl --set art.pxl '3,4=K' '0-9,4=.' '12-15,20-23=H'
```

Each edit has the form `x,y=K`:

- `x` and `y` are a 0-based coordinate, as in the grid view, or an inclusive
  range `a-b`.
- `K` is one palette key. Any key is valid, including `=` and `,`.
- `x,y=K` sets one pixel, `x0-x1,y=K` sets a run in one row, and
  `x0-x1,y0-y1=K` sets a rectangle.

Put each edit in single quotes, because some keys are special in the shell.
The edits apply from left to right, so a later edit wins where edits
overlap. If an edit is outside the image or uses a key that is not in the
palette, `pxl` writes nothing, and the error names the edit:

```text
error: art.pxl: edit `40,3=K`: x is outside the image; use x from 0 to 39
```

After a success, `pxl` prints the number of changed pixels to stderr. It
writes the file like the editor does: each palette line as
`<key> - #RRGGBBAA`, and one blank line before the pixel rows. Use `-o` to
write to a different file, or `-o -` to see the result without a change to the
file.

### Terminal editor

Use `-e` to edit a file in the terminal:

```sh
pxl -e examples/basketball.pxl
```

The screen has a canvas at the left and the palette at the right. Each pixel
is two terminal columns wide, so that it is about square. Transparent pixels
show a checkerboard. Use a terminal with 24-bit color, because other
terminals show approximate colors. The file always keeps the exact colors.

| Action                      | Mouse                                   | Key                     |
| --------------------------- | --------------------------------------- | ----------------------- |
| Draw with the selected color | Click or drag on the canvas            |                         |
| Select a color              | Click a palette row                     | `Tab`, `Shift+Tab`      |
| Pick a color from the image | Right-click a pixel                     | `i` (the pixel below the pointer) |
| Add a color                 | **Add**                                 | `a`                     |
| Edit key, color, or label   | **Edit**                                | `e`, `Enter`            |
| Delete a color              | **Delete**                              | `d`, `Delete`           |
| Scroll a large image        | Wheel; `Shift`+wheel scrolls sideways   | Arrow keys              |
| Save the file               | **Save**                                | `Ctrl+S`                |
| Quit                        | **Quit**                                | `q`, `Ctrl+Q`           |

In the Add and Edit dialogs, type the color as `#RRGGBB` or `#RRGGBBAA`.
`#RRGGBB` gets the alpha `FF`. The label is optional, with a maximum of 32
chars. A blank label removes the label. The status bar shows the label of the
selected color. Use `Tab` to go to the next field, `Enter` to accept, and
`Esc` to cancel. A new color for an entry changes all the pixels that use it.

If pixels use a color that you delete, the dialog asks for a replacement
key. The pixels change to that key. The dialog proposes the first fully
transparent color.

After each save, a dialog gives the result: the number of bytes and the path,
or the cause of the failure. The editor reads the file again after it writes
it, so a success means that the file has the new text. Close the dialog with
**OK**, `Enter`, or `Esc`.

Quit always asks for confirmation:

- With no unsaved changes, select **Quit** (`y`) or **Cancel** (`n`, `Esc`).
- With unsaved changes, select **Save and quit** (`s`), **Discard** (`d`), or
  **Cancel** (`Esc`). If the save fails, the editor stays open and shows the
  failure. After a save and quit, `pxl` prints the save result in the
  terminal.

When the editor saves, it writes each palette line as `<key> - #RRGGBBAA`,
with uppercase digits and the label if the entry has one, and one blank line
before the pixel rows. The palette order and the pixel data do not change.

## Tasks

| Task             | Description                         |
| ---------------- | ----------------------------------- |
| `mise run build` | Build the debug binary. Add `--release` for a release build. |
| `mise run test`  | Run all tests.                      |
| `mise run lint`  | Check the format and run clippy. Clippy denies all warnings, including the `pedantic` group. |
| `mise run fmt`   | Format the source code.             |
| `mise run install` | Build the release binary and install `pxl` in `$HOME/.local/bin`. |
| `mise run render -- <args>` | Render a `.pxl` file to PNG. |

The clippy settings are in two files:

- `Cargo.toml`, in the `[lints.clippy]` table: denies the `pedantic` group,
  `cognitive_complexity`, and `excessive_nesting`.
- `clippy.toml`: sets the limits. A function can have at most 40 lines, 4
  arguments, 4 levels of nesting, and a cognitive complexity of 7.

Fix the code to obey a lint. Use `#[allow]` only if the fix makes the code
worse, and write a comment that gives the reason.

## Agent skill

`.agents/skills/pxl-art/` is an agent skill for drawing `.pxl` art. The skill
tells the agent to work like a pixel artist, in short passes of draw, look,
and refine:

1. The agent edits the `.pxl` file directly: whole rows for new shapes, and
   `pxl --set` for small fixes. It does not generate pixels with a script.
2. After each pass, the agent renders the clean view and the grid view and
   looks at them.
3. The agent writes a list of defects with coordinates, then corrects the
   worst defect first.

The stages are brief, layout plan, palette, canvas, silhouette, lines,
shading, detail, cleanup, and final check. `references/TECHNIQUES.md` lists
the principles of good pixel art and the common defects.

If you give a PNG or JPG of pixel art to change or improve, the agent
converts it with `pxl -c` first. For a PNG, it finds the exact scale with a
round-trip check. Then it changes or improves the art with the same loop. A
reference image for new art is not converted. The agent only looks at it.

You can give options in the request, in words or as `name: value` pairs:

```text
Draw a treasure chest. quality: polished, style: 16-bit, colors: 12, size: 32x32
```

| Option       | Values                                                       |
| ------------ | ------------------------------------------------------------ |
| `quality`    | `sketch`, `clean` (default), `polished`                      |
| `style`      | `1-bit`, `gameboy`, `8-bit`, `16-bit` (default), `modern`, `flat` |
| `colors`     | Maximum palette size                                         |
| `palette`    | `free`, `1-bit`, `gameboy`, `pico-8`, or a list of hex colors |
| `size`       | `WxH`                                                        |
| `outline`    | `full`, `selective`, `none`                                  |
| `shading`    | `flat`, `cel`, `soft`, `dither`                              |
| `light`      | `top-left` (default), `top`, `top-right`, `left`, `right`, `front` |
| `background` | `transparent` (default) or a hex color                       |
| `view`       | `front`, `three-quarter`, `side`, `top-down`, `isometric`    |

`quality` sets how many stages and critique passes the agent does. Each style
sets default values for the other options. An option that you give has
priority over the style.

If you give no options, the agent interviews you first. It asks about the
style, quality, size, and view in one round, and it recommends a default for
each. Tell the agent "use the defaults" to skip the interview.

### Install the skill

The skill runs the `pxl` command, so install `pxl` first:

```sh
mise run install
```

Inside this repository, most harnesses find the skill with no setup. To use
the skill in other projects, link it into the skills directory of your
harness. A link keeps the skill up to date when you pull changes. Run the
commands from the root of this repository.

| Harness                          | Skills directory    | Reads `.agents/skills/` in this repository |
| -------------------------------- | ------------------- | ------------------------------------------ |
| Codex, Gemini CLI, OpenCode, omp | `~/.agents/skills/` | Yes                                        |
| Claude Code                      | `~/.claude/skills/` | No                                         |

For Codex, Gemini CLI, OpenCode, omp, and other harnesses that read the
shared `~/.agents/skills/` directory:

```sh
mkdir -p ~/.agents/skills
ln -s "$PWD/.agents/skills/pxl-art" ~/.agents/skills/pxl-art
```

For Claude Code:

```sh
mkdir -p ~/.claude/skills
ln -s "$PWD/.agents/skills/pxl-art" ~/.claude/skills/pxl-art
```

To use Claude Code only in this repository, link the skill into the
project. Use a relative link, so that the link works in all clones:

```sh
mkdir -p .claude/skills
ln -s ../../.agents/skills/pxl-art .claude/skills/pxl-art
```

For a harness that uses a different directory, link or copy the `pxl-art`
directory into its skills directory. The skill uses only the standard `name`
and `description` fields, so any harness that supports the
[Agent Skills](https://agentskills.io) format can load it.

Start a new session after you install the skill. Most harnesses read the
skills only when a session starts.
