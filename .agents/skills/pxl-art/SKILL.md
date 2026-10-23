---
name: pxl-art
description: Draw or improve pixel art in the .pxl text format with the loop of a pixel artist - draw, look, refine. Accepts options for quality, style, color count, palette, size, outline, shading, light, background, and view. If the request gives no options, interviews the user about them first. Converts a PNG or JPG of pixel art to .pxl, so that the art can be changed. Use this skill when the user asks you to create, change, or improve a .pxl file or pixel art, or to convert a pixel art image to .pxl.
---

# Pixel art in `.pxl`

Work like a pixel artist. Put each pixel on purpose. Look at the result. Correct it.
Do many short passes. Each pass has one goal.

## Rules

1. Write and change the `.pxl` file directly with your file tools: `write` for a new file, `edit` for changes.
2. NEVER generate pixel data with a script, a program, or shape formulas (ellipses, polygons, noise, gradients).
   Formula shapes look stiff and generic. They also remove the pixel-level decisions that make pixel art good.
   `pxl -c` is not generation: it copies the pixels of existing pixel art. See [Start from an image](#start-from-an-image).
3. Render and look at the image after every pass. NEVER deliver art that you did not look at after the last change.
4. NEVER deliver the first draft. Do the critique passes that the [quality level](#quality-levels) requires.
5. Keep every pixel row at exactly the canvas width. The parser rejects a row with a different width.
6. Obey the [options](#options). NEVER use more colors than the color limit.
7. Get the [critic reviews](#critic-review) that the quality level requires. NEVER give the critic your plan or your notes.

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

For a change to an existing `.pxl` file, or to a pixel art image that you [convert](#start-from-an-image), the file gives the size and the palette. Do not ask about them. Ask only about the change, if the request does not make it clear.

### Quality levels

`quality` sets how pretty the result is. It controls the stages and the number of passes.

| Quality    | Stages                                                     | Minimum critique passes                                                  | Result                                                                                      |
| ---------- | ---------------------------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `sketch`   | 1-6 and 10. Flat base colors, no shading.                  | 1, after the silhouette.                                                 | Correct proportions, a clear silhouette, and clean lines.                                    |
| `clean`    | All stages.                                                | 1 for each stage.                                                        | One light direction, 2-3 tones for each material, and no line defects.                       |
| `polished` | All stages.                                                | 1 for each stage, and 2 for shading and for cleanup. Continue until the critique checklist finds no defect. | All the [techniques](references/TECHNIQUES.md) that the style allows: hue-shifted ramps, selective outline, anti-aliasing, texture, reflected light, and a strong focal point. |

Each quality level also requires [critic reviews](#critic-review) at fixed points.

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
If the user asks for a strict retro look, use the [hardware limits](references/TECHNIQUES.md#hardware-limits) of the `8-bit` or `gameboy` style.

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

If `pxl` is not on `PATH`, tell the user to install it: run `mise run install` in the pixel-rs repository. Inside the pixel-rs repository, you can use `cargo run --quiet --` in place of `pxl`. If `pxl` rejects an option in this skill, for example `-c`, the installed `pxl` is old. Tell the user to run `mise run install` again.

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

Do not use `pxl -p` or `pxl -e`. `-p` prints terminal color codes, which you cannot see as an image. `-e` is an interactive editor for the user.

## Start from an image

Use this section only if the image is the art itself: the user wants to change, improve, or convert that image.

Do not convert a reference image. A reference shows the subject, the style, or the colors for new art. Look at it with `read`, and draw the new art with the normal stages. If the role of the image is not clear, ask the user.

CAUTION: Convert only pixel art. A photo, a drawing, or a smooth (anti-aliased) resize of pixel art becomes noise, with hundreds of colors. Then tell the user, and draw the subject with the normal stages. Use the image as a reference.

1. Get the image size:

   ```sh
   file <image>
   ```

2. Look at the image with `read`. Estimate the scale: the size in image pixels of one art pixel, for example 1, 2, 4, or 8. The scale must divide the width and the height.
3. For a PNG, find the exact scale. Test each candidate scale `N`, from the largest to the smallest. The first scale that prints `exact` is the correct scale:

   ```sh
   pxl -c -s N -o /tmp/<name>-sN.pxl <image> && pxl -s N -o - /tmp/<name>-sN.pxl | pxl -c -o - - | cmp -s - <(pxl -c -o - <image>) && echo "scale N is exact"
   ```

   The command converts at scale `N`, renders the result at scale `N`, and compares it with the image. Scale 1 is always exact. If only scale 1 is exact, but the image looks scaled, the art is not on an exact grid. Then tell the user. The image possibly has a border, or a resize that is not a whole number.
4. For a JPG, the exact check does not work. Choose the scale from the image. Convert, render the result at the same scale, and compare the two images by eye.
5. Convert the image to the `.pxl` file:

   ```sh
   pxl -c -s N -o <file>.pxl <image>
   ```

6. Read the `.pxl` file. Fully transparent pixels have the key `.`. The other keys are `A`-`Z`, `a`-`z`, `0`-`9`, and then punctuation, in order of color frequency.
7. For a JPG, find palette colors that are almost equal. Merge the rare color into the frequent color. See [Merge or rename a key](#merge-or-rename-a-key).
8. Render the grid view and the clean view. Compare them with the image.
9. Continue with the request. For a change or an improvement, use the stages from the [layout plan](#2-layout-plan). Keep the parts of the art that the request does not change.

### Merge or rename a key

A merge or a rename of a key changes all pixel rows. This is a mechanical change, not generation, so you can use one command for it. `P` is the file line of row 0.

1. Change the key in the pixel rows. For example, change `x` to `K`:

   ```sh
   sed -i 'P,$ y/x/K/' <file>.pxl
   ```

2. For a merge, remove the palette line of `x`. For a rename, change the key on the palette line of `x`. Make sure that no other palette line uses the new key.
3. A merge changes `P`. Calculate `P` again.
4. Render the file, and make sure that it parses.

Do not use `/` or `\` as a key in the `sed` command.

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
   At a point that requires a [critic review](#critic-review), get the review now, and add its defects to your list.
6. **Decide.** If a defect is easy to see, start the next pass with it. If not, go to the next stage.

Fix large problems before small problems, in this order:
proportion, silhouette, color, shading, detail, cleanup.
A good detail cannot correct a bad proportion.

Be honest in the critique. Compare your drawing with the work of a skilled pixel artist, not with your last pass.

## Critic review

You know what you intended to draw, so you see your intent and not the pixels. A critic is a new subagent that sees only the pixels.

| Quality    | Critic reviews                                               |
| ---------- | ------------------------------------------------------------ |
| `sketch`   | None.                                                        |
| `clean`    | After the silhouette (stage 5), and in the final check.      |
| `polished` | After the silhouette (stage 5), after shading (stage 7), and in the final check. |

Do these steps for each review:

1. Render the clean view, the grid view, and the 1x view of the current pass.
2. Start a new subagent with the subagent tool of your harness, for example `Agent` or `task`. Give it the [critic prompt](#critic-prompt). Use a new subagent for each review. A subagent that saw an earlier pass compares the art with that pass.
3. Give the critic only the render paths, the subject, the canvas size, and the resolved options. Do not give it your layout plan, your critiques, your pass history, or the changes that you intended.
4. Examine each defect from the critic on the grid view. Fix each defect that you can see. Reject a defect only if the grid view shows that it is not there, and write the reason in your notes.
5. Fix the defects in the usual order: proportion, silhouette, color, shading, detail, cleanup.

In the final check, do the review again after your fixes. Stop when the critic writes `no visible defects`, or after 3 reviews. Then tell the user about each defect that remains.

If you cannot start a subagent, review the art yourself. Look at the renders before you read your notes, and write the defects before you compare them with your plan. Tell the user that no independent critic reviewed the art.

### Critic prompt

Replace each `<...>` field. For the checklist, copy the [Critique checklist](#critique-checklist) without the questions about the layout plan and the last fix.

```text
You are a strict pixel art critic. Review the pixel art in these images. Do not change any file.

Subject: <subject>
Canvas: <W>x<H>
Options: <resolved options in one line>
Images. Look at each image with your image read tool:
- Clean view: <path>
- Grid view: <path>. The labels at the top give x. The labels at the left give y. Both start at 0. A dark line is at each multiple of 10. A checkerboard shows the transparent pixels.
- Actual size: <path>

Compare the art with the work of a skilled pixel artist in this style. Do not praise. Report only defects.
Also find each part of the subject that is wrong, missing, or not recognizable.

Checklist:
<checklist>

Answer in this format:
1. Clear at actual size: yes or no, and the reason.
2. Defects, worst first. One line for each defect: <part>, x <range>, y <range>: <defect>. <fix>.
If you find no defect that is easy to see, write "no visible defects" in place of the list.
```

## Stages

Do the stages that the [quality level](#quality-levels) requires. [references/TECHNIQUES.md](references/TECHNIQUES.md) lists the techniques for each stage.

### 1. Brief

Find the subject and the mood. [Resolve the options](#options). If the request gives no options, [interview the user](#interview) first.
If the request gives a PNG or JPG of pixel art to change or improve, [convert it](#start-from-an-image) first. If the image is a reference, only look at it.
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
Use the rules of the view in [Views](references/TECHNIQUES.md#views). For a canvas of 32 px or less, use [Small sprites](references/TECHNIQUES.md#small-sprites).

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
3. Get the [critic reviews](#critic-review) that the quality level requires, and fix the defects.
4. Tell the user the file path, the canvas size, the color count, the resolved options, and the command to render the file.
5. Tell the user that `pxl -p <file>.pxl` shows the art in the terminal, and that `pxl -e <file>.pxl` opens it in the terminal editor for manual changes.

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
