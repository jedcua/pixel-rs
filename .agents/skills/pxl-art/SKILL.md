---
name: pxl-art
description: Draw or improve pixel art in the .pxl text format with the loop of a pixel artist - draw, look, refine. Accepts options for quality, style, color count, palette, size, outline, shading, light, background, and view. If the request gives no options, interviews the user about them first. Use this skill when the user asks you to create, change, or improve a .pxl file or pixel art.
---

# Pixel art in `.pxl`

Work like a pixel artist. Put each pixel on purpose. Look at the result. Correct it.
Do many short passes. Each pass has one goal.

## Rules

1. Write and change the `.pxl` file directly with your file tools: `write` for a new file, `edit` for changes.
2. NEVER generate pixel data with a script, a program, or shape formulas (ellipses, polygons, noise, gradients).
   Formula shapes look stiff and generic. They also remove the pixel-level decisions that make pixel art good.
3. Render and look at the image after every pass. NEVER deliver art that you did not look at after the last change.
4. NEVER deliver the first draft. Do the critique passes that the [quality level](#quality-levels) requires.
5. Keep every pixel row at exactly the canvas width. The parser rejects a row with a different width.
6. Obey the [options](#options). NEVER use more colors than the color limit.

## Options

The user can give options in the request, in words or as `name: value` pairs:

```text
Draw a treasure chest. quality: polished, style: 16-bit, colors: 12, size: 32x32
```

| Option       | Values                                                                 | Default                          |
| ------------ | ---------------------------------------------------------------------- | -------------------------------- |
| `quality`    | `sketch`, `clean`, `polished`                                          | `clean`                          |
| `style`      | `1-bit`, `gameboy`, `8-bit`, `16-bit`, `modern`, `flat`                | `16-bit`                         |
| `colors`     | Maximum palette size, from 2. A fully transparent background does not count. | From the style              |
| `palette`    | `free`, `1-bit`, `gameboy`, `pico-8`, or a list of hex colors          | From the style                   |
| `size`       | `WxH`, for example `32x32`                                             | From the subject (see [Brief](#1-brief)) |
| `outline`    | `full`, `selective`, `none`                                            | From the style                   |
| `shading`    | `flat`, `cel`, `soft`, `dither`                                        | From the style                   |
| `light`      | `top-left`, `top`, `top-right`, `left`, `right`, `front`               | `top-left`                       |
| `background` | `transparent` or a hex color                                           | `transparent`                    |
| `view`       | `front`, `three-quarter`, `side`, `top-down`, `isometric`              | The view that shows the subject best |

Resolve the options in the [Brief](#1-brief) stage:

1. If the request gives no options, [interview the user](#interview). Then use each answer as an option that the user gives.
2. Use each option that the user gives.
3. Use the style preset for each option that the user does not give. Then use the defaults in the table.
4. An option that the user gives has priority over the style preset.
5. If `palette` is a preset or a list, use only those colors. Then `colors` cannot be more than the palette size.
6. If two options conflict, choose the option that the user stated more clearly. Tell the user about the conflict.
7. Before you draw, tell the user the resolved options in one line.

### Interview

Do the interview only if the request gives no options, in words or as `name: value` pairs.
If the request gives one or more options, do not interview. Resolve the other options as usual.

1. Ask all questions in one round. If your harness has a question tool (for example `ask` or `AskUserQuestion`), use it. If not, ask in a short numbered list.
2. Ask about these options, in this order: `style`, `quality`, `size`, `view`.
3. Ask about `palette`, `colors`, `background`, or `light` only if the subject makes them important. An example is a sprite for a game with a fixed palette.
4. For each question, give 2-5 options with a short description of the effect. Mark the default as the recommended option.
5. For `size`, recommend the size for the subject type from the [Brief](#1-brief) table.
6. Do not ask about the subject if the request states it. Ask about the subject only if the request does not state it.

Do not interview in these conditions. Use the defaults, and tell the user your choices:

- The user tells you to choose, for example "surprise me" or "use the defaults".
- You cannot ask the user, for example because you run as a subagent or in a non-interactive session.
- The user does not answer a question. Use the default for that option only.

For a change to an existing `.pxl` file, the file gives the size and the palette. Do not ask about them. Ask only about the change, if the request does not make it clear.

### Quality levels

`quality` sets how pretty the result is. It controls the stages and the number of passes.

| Quality    | Stages                                                     | Minimum critique passes                                                  | Result                                                                                      |
| ---------- | ---------------------------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `sketch`   | 1-6 and 10. Flat base colors, no shading.                  | 1, after the silhouette.                                                 | Correct proportions, a clear silhouette, and clean lines.                                    |
| `clean`    | All stages.                                                | 1 for each stage.                                                        | One light direction, 2-3 tones for each material, and no line defects.                       |
| `polished` | All stages.                                                | 1 for each stage, and 2 for shading and for cleanup. Continue until the critique checklist finds no defect. | All the [techniques](references/TECHNIQUES.md) that the style allows: hue-shifted ramps, selective outline, anti-aliasing, texture, reflected light, and a strong focal point. |

Quality does not add techniques that the style excludes. A `polished` `flat` image has flat colors with perfect lines and shapes.

### Style presets

| Style     | Colors | Palette   | Outline     | Shading  | Anti-aliasing | Dithering                 |
| --------- | ------ | --------- | ----------- | -------- | ------------- | ------------------------- |
| `1-bit`   | 2      | `1-bit`   | `none`      | `dither` | No            | Yes, the only way to make mid-tones |
| `gameboy` | 4      | `gameboy` | `full`      | `cel`    | No            | Some, for large gradients |
| `8-bit`   | 4-8    | `free`    | `full`      | `cel`    | No            | Some                      |
| `16-bit`  | 12-24  | `free`    | `selective` | `soft`   | Yes, a little | Rare                      |
| `modern`  | 16-48  | `free`    | `selective` | `soft`   | Yes           | No                        |
| `flat`    | 4-12   | `free`    | `full`      | `flat`   | No            | No                        |

In `8-bit` style, use at most 3 colors for each object, plus the outline. Use large, simple shapes.

### Shading types

- `flat`: one color for each material. You can add one shadow tone only for cast shadows.
- `cel`: 2-3 tones for each material, with hard edges between the tones.
- `soft`: 3-5 tones for each material, with mid-tones at the edge between light and shadow.
- `dither`: patterns of two colors make the tones between them.

### Palette presets

| Preset    | Colors                                                                                                                                                                                                                       |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `1-bit`   | `#000000`, `#FFFFFF`                                                                                                                                                                                                         |
| `gameboy` | `#0F380F`, `#306230`, `#8BAC0F`, `#9BBC0F`                                                                                                                                                                                   |
| `pico-8`  | `#000000`, `#1D2B53`, `#7E2553`, `#008751`, `#AB5236`, `#5F574F`, `#C2C3C7`, `#FFF1E8`, `#FF004D`, `#FFA300`, `#FFEC27`, `#00E436`, `#29ADFF`, `#83769C`, `#FF77A8`, `#FFCCAA`                                                     |

Write each preset color with the alpha `FF` in the `.pxl` file, for example `#0F380FFF`.
With a preset, you can use a subset of its colors.

## The format

```pxl
. - #00000000
K - #2A1A33FF
H - #5B4B8AFF

..KK..
.KHHK.
..KK..
```

- The palette comes first. Each line is `<char> - #RRGGBBAA`. The key is one visible char.
- One blank line comes next.
- Then the pixel rows. Each char is one pixel. All rows have the same width.

## Tools

Use the `pxl` command. Read each PNG with the `read` tool to look at it.

If `pxl` is not on `PATH`, tell the user to install it: run `mise run install` in the pixel-rs repository. Inside the pixel-rs repository, you can use `cargo run --quiet --` in place of `pxl`.

1. Grid view, to find coordinates:

   ```sh
   pxl -g -s <grid-scale> -o /tmp/<name>-p<pass>-grid.png <file>.pxl
   ```

2. Clean view, for the overall impression:

   ```sh
   pxl -s <clean-scale> -o /tmp/<name>-p<pass>.png <file>.pxl
   ```

3. Actual size, for readability (the "squint test"):

   ```sh
   pxl -s 1 -o /tmp/<name>-p<pass>-1x.png <file>.pxl
   ```

4. Width check of all rows. The parser reports only the first bad row. This command lists all of them. `P` is the file line of row 0, and `W` is the canvas width (see [Coordinates and file lines](#coordinates-and-file-lines)):

   ```sh
   awk -v p=P -v w=W 'NR >= p && length($0) != w { print "line " NR ", y " NR - p ": width " length($0) }' <file>.pxl
   ```

Put the pass number in each image name (`<pass>`). The `read` tool can return a cached image for a file name that it read before, and then you look at an old pass.

Keep the images at about 1000 px or less. The image viewer reduces larger images, and then you cannot see single pixels.

| Canvas width | Grid scale | Clean scale |
| ------------ | ---------- | ----------- |
| 16           | 32         | 32          |
| 32           | 24         | 24          |
| 64           | 12         | 12          |
| 100          | 8          | 8           |

The grid view has these features:

- The labels at the top give `x`. The labels at the left give `y`. Both start at 0.
- A dark line is at each multiple of 10. A light line is between all other cells.
- A checkerboard shows the transparent pixels.

## Coordinates and file lines

- Pixel `(x, y)` is char `x + 1` (1-based) of pixel row `y`.
- Pixel row `y` is on file line `P + y`. `P` is the file line of row 0.
  With one blank line after the palette, `P` = (palette line count) + 2.
- Find `P` one time with `read`, and write it down. Do the calculation again if the palette changes size.
- Parser errors give the file line and the 1-based column.

## Edit technique

- Change whole rows. To change a region, replace the block of rows that contains the region.
- Read the current rows with `read` before you change them. Do not type rows from memory.
- To change some pixels in a row, copy the current row and change only those chars.
- Before you type a new row, make a run plan. Give each run its x range and key. Then make sure that the runs fill the width with no gap and no overlap:

  ```text
  y=23 (width 40): x0-12 . | x13-15 K | x16-27 H | x28-30 K | x31-39 .
  -> .............KKKHHHHHHHHHHHHKKK.........
  ```

- Align your attention to the blocks of 10 cells from the grid view. Count each block of 10 chars, not the full row.
- For a symmetric subject, draw the left half. Make the right half as the left half in reverse order. Then add asymmetry where the subject needs it, for example hair or light.
- After each edit, run the width check and then the renders. Fix all rows with a bad width before you do other work.
- Expect typing errors in long rows. In a trial, 6 of 32 rows had a bad width after the first full write. The width check finds all of them in one run.

## The loop

Do these steps in each pass:

1. **Goal.** Write one goal for the pass, for example "fix the jaw line".
2. **Draw.** Change only the rows for that goal.
3. **Render.** Make the clean view and the grid view.
4. **Look.** Examine the clean view first, for the overall impression. Then use the grid view to find coordinates. Use the 1x view to check readability.
5. **Critique.** Write a list of concrete defects. Give each defect a location and a fix:
   `Left eye, x 30-37, y 52-60: one row lower than the right eye. Move it up one row.`
   Use the [Critique checklist](#critique-checklist). Write the worst defect first.
6. **Decide.** If a defect is easy to see, start the next pass with it. If not, go to the next stage.

Fix large problems before small problems, in this order:
proportion, silhouette, color, shading, detail, cleanup.
A good detail cannot correct a bad proportion.

Be honest in the critique. Compare your drawing with the work of a skilled pixel artist, not with your last pass.

## Stages

Do the stages that the [quality level](#quality-levels) requires. [references/TECHNIQUES.md](references/TECHNIQUES.md) lists the techniques for each stage.

### 1. Brief

Find the subject and the mood. [Resolve the options](#options). If the request gives no options, [interview the user](#interview) first.
After the options are resolved, ask the user again only if a missing fact changes the result a lot. If not, choose, and tell the user your choice.

If the user does not give `size`, use the size for the subject type:

| Subject type                | Size  |
| --------------------------- | ----- |
| Icon or small item          | 16x16 |
| Large item or game sprite   | 32x32 |
| Character or portrait       | 64x64 |

A small canvas is easier to make good. For a canvas larger than 64 px, the loop is the same. Plan more passes, and work on one region at a time.

### 2. Layout plan

Before you draw, write a landmark plan in coordinates. Include the center line, the top and bottom of the subject, and the positions and sizes of the main parts.
Use the real proportions of the subject in the chosen style and view. Leave a margin of at least 1 px around the subject.

### 3. Palette

- Use the colors of the palette preset, or make a palette within the color limit.
- Make a ramp for each material. The number of tones comes from the [shading type](#shading-types).
- For a `free` palette, shift the hue along each ramp. See [Color](references/TECHNIQUES.md#color).
- Choose keys that are easy to remember. For example, use an uppercase letter for a base color and the lowercase letter for its shadow: `H` hair, `h` hair shadow. Use `.` for the background.
- Do not use keys that look alike, such as `0` and `O`, or `1`, `l`, and `I`.
- For a transparent background, use `. - #00000000`.
- Count the colors. Do not count a fully transparent background.

### 4. Canvas

Write the palette, one blank line, and then the rows of background chars, at the full canvas size.
Render it, and make sure that the size is correct.

### 5. Silhouette

Fill each large mass with one flat base color. Do not add detail yet.
Look at the 1x view. The subject must be clear from its shape alone.
Correct the proportions now. A change is easy now and difficult later.

### 6. Lines and outline

Draw the outline that the `outline` option gives:

- `full`: a dark outline around the subject and between the main parts.
- `selective`: like `full`, but on the lit side the outline is the darkest tone of the material.
- `none`: no outline. Make the shapes clear with value contrast only.

Make all lines clean. See [Lines](references/TECHNIQUES.md#lines).

### 7. Color and shading

Shade with the [shading type](#shading-types) and the `light` direction. See [Light and shading](references/TECHNIQUES.md#light-and-shading).
For `flat` shading, use this stage only to correct the base colors.

### 8. Detail and focus

Put most contrast and detail at the focal point. Keep the other areas simpler.
See [Detail and texture](references/TECHNIQUES.md#detail-and-texture).

### 9. Cleanup

Remove orphan pixels and correct line defects.
Add anti-aliasing and dithering only if the style allows them. See [Edges](references/TECHNIQUES.md#edges-and-anti-aliasing) and [Dithering](references/TECHNIQUES.md#dithering).

### 10. Final check

1. Render at 1x and at the delivery scale.
2. Compare the result with the brief, the layout plan, and the resolved options. Count the colors in the palette.
3. Tell the user the file path, the canvas size, the color count, the resolved options, and the command to render the file.

## Critique checklist

- Is the subject clear in the 1x view?
- Do the proportions agree with the layout plan?
- Are the parts symmetric where they must be? Compare the left and right sides by coordinates.
- Is the silhouette clear against the background?
- Do the lines have jaggies, doubles, or gaps?
- Does all shading use one light direction? Do you see pillow shading or banding?
- Do adjacent areas have enough value contrast? Is the color count within the limit?
- Are there orphan pixels or noise?
- Does the focal point have the most contrast?
- Does the result agree with the style and the other options?
- Did the last fix make a new problem near it?

## References

- [references/TECHNIQUES.md](references/TECHNIQUES.md): the important techniques and principles of good pixel art, with the common defects.
