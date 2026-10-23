# Pixel art techniques

These are the important principles and techniques of good pixel art. They apply to all subjects.
Use only the techniques that the style and the options allow.

## Readability

- **Silhouette first.** The subject must be clear from its shape alone, in one flat color, at 1x.
- **Every pixel counts.** At small sizes, one pixel can change an expression or a form. Put each pixel on purpose.
- **Exaggerate the key features.** Make the parts that identify the subject larger than real proportions. Remove the parts that do not help.
- **Value contrast before hue contrast.** Adjacent areas must differ in brightness, not only in hue. Imagine the image in grayscale: the forms must stay clear.
- **Use negative space.** The background between parts also makes shapes. Keep gaps open where they help to separate parts.
- **Detail must agree with the size.** Do not draw details that are smaller than one pixel. Suggest them with one pixel or one cluster.

## Small sprites

- **Draw the idea at 16x16.** At 16x16, show the silhouette and one or two key features, not the full object. A clear 16x16 silhouette is better than a detailed 32x32 that does not read.
- **Few colors at 16x16.** Use 3-4 colors for a 16x16 sprite. If the subject needs a fifth color, think about 32x32.
- **Big heads.** For a small character, make the head 1/3 to 1/2 of the sprite height. The large head shows the expression. Shorten the torso to make space for the eyes.
- **Eyes.** At a character height of 16 px, an eye is 1 px wide and 2 px high, with no mouth. At 32 px, an eye can be 2x2. Make the eyes high contrast.
- **Consistent limbs.** Keep the thickness and the length of each limb the same in all poses and views. At this size, one pixel changes the result.
- **Clear extremities.** Hands, feet, and the ends of the hair must have a value that is different from the body and the background. A mid-value extremity disappears into the silhouette.

## Lines

- **1 px lines.** Make outlines and inner lines 1 px wide. Make a line thicker only on purpose, for example for weight on the shadow side.
- **No doubles.** A 1 px line touches the next pixel only at one side or at one corner. In an L-shaped corner, remove the corner pixel.
- **Smooth curves.** The run lengths of a curve change in order, for example 5, 3, 2, 1, 1, 2, 3, 5. A sudden change, such as 3, 1, 3, makes a jaggy.
- **Clean slopes.** A straight slope uses runs of equal length: 1:1, 2:1, 3:1, 4:1. A mix, such as 2, 1, 3, looks bumpy. A mix is correct only if it repeats, such as 2, 1, 2, 1.
- **No gaps.** An outline has no holes, unless a gap shows light or a separation.
- **Inner lines are lighter.** Lines inside the silhouette are usually lighter than the outer outline, or they are a dark tone of the material.

Examples (`K` is a line pixel):

```text
Smooth curve      Jaggy curve       Double            Clean corner
KKKKK.......      KKK.........      KKK...            KK....
.....KKK....      ...K........      ..K...            ..K...
........KK..      ....KKK.....      ..KKK.            ...KK.
..........K.      .......K....
```

## Clusters

- **Think in clusters.** A cluster is a group of pixels of one color. Clear clusters make clear forms.
- **No orphans.** A single pixel that is different from all its neighbors looks like noise. Use it only on purpose, for example for a highlight or a star.
- **Clusters follow the form.** The shape of a shading cluster shows the shape of the surface below it.
- **Consistent cluster size.** Clusters of similar size in one area make a consistent texture and surface.

## Color

- **Limited palette.** Few colors make a consistent image. Use the smallest palette that shows the forms and materials.
- **Ramps.** Each material has a ramp of tones from dark to light.
- **Hue shift.** Move the hue along each ramp. Shadows go toward blue or purple. Highlights go toward yellow or orange. A ramp that changes only in brightness looks dull. Shift the hue by 20 degrees or less between adjacent tones.
- **Brightness.** Increase the brightness at each step of the ramp. Start above 0, unless the ramp must end in black.
- **Saturation.** Make the mid-tones the most saturated. Make the darkest and lightest tones less saturated. A bright tone with high saturation hurts the eyes.
- **Neutral ramp.** For grays, stone, and metal, use a ramp with low saturation. Several materials can share it.
- **Warm light, cool shadow.** Under a warm light, the shadows are cool, and the opposite is also true.
- **Shared colors.** Use one color in more than one ramp where possible. Shared colors make the palette consistent and save colors.
- **No pure black or white.** Use a very dark, saturated color for outlines and deep shadows. Use a slightly warm or cool color for highlights. A style or a palette preset can require pure colors.
- **Enough steps between tones.** Two tones that are almost the same waste a color and make the image look muddy.

Example ramps with hue shift:

| Material | Shadow    | Base      | Light     | Highlight |
| -------- | --------- | --------- | --------- | --------- |
| Skin     | `#B5655E` | `#E9A487` | `#FAD2B4` | `#FFF0DC` |
| Foliage  | `#1F4D45` | `#3B7D3A` | `#76B045` | `#C2DE6E` |

## Light and shading

- **One light source.** Choose one light direction and use it for all parts.
- **Shade the form.** Think of each part as a 3D shape: a sphere, a cylinder, or a box. The light side and the shadow side follow that shape.
- **No pillow shading.** Do not shade from the outline to the center. That shading follows the outline, not the light, and the image looks flat.
- **No banding.** Steps of different tones must not align next to each other. Change the width of each tone band, or merge two tones.
- **Soft terminator.** On a round form, put a mid-tone at the edge between light and shadow. On a hard edge, such as the corner of a box, use no mid-tone.
- **Cast shadows.** A part that blocks the light makes a shadow on the parts below it. Cast shadows show the positions of the parts.
- **Ambient occlusion.** Creases and contact points get darker, because little light gets into them.
- **Reflected light.** The shadow side can be a little lighter near its edge, because of light from the ground. Keep it darker than the light side.
- **Small highlights.** A highlight is small and sharp. A large highlight makes a surface look wet or flat.
- **Materials.** Metal and glass have high contrast and sharp highlights. Cloth and skin have low contrast and soft transitions.

## Edges and anti-aliasing

- **Selective outline.** On the lit side, use the darkest tone of the material for the outline, not the outline color. This makes the form softer and keeps the silhouette clear.
- **Manual anti-aliasing.** Put a mid-tone pixel in the inner corner of a hard step. This makes a curve or a slope look smoother.
- **Use anti-aliasing a little.** Too much anti-aliasing makes the image look blurry.
- **No anti-aliasing at a transparent edge.** The background color is unknown, so a mid-tone at the edge can look wrong on it.

```text
Without           With (1 is a mid-tone)
KKKK....          KKKK1...
....KKKK          ...1KKKK
```

## Dithering

- **Purpose.** Dithering makes tones between two colors with a pattern. Use it when the palette is too small for a smooth gradient.
- **Regular patterns.** Use a checkerboard for 50 %, and a sparser grid for 25 % and 75 %. Random dithering looks like noise.
- **Large areas only.** Dithering in a small area or at a small size looks like noise.
- **Transition zones.** Put the dither in the zone between two tones, not over a full area.

## Detail and texture

- **Focal point.** Put the most contrast and detail at the focal point. Keep the other areas simpler. These areas give the eye a rest.
- **Suggest texture.** A few clusters suggest a texture, such as wood grain, fur, or stone. A full pattern of single pixels looks like noise.
- **Consistent detail.** Keep the detail level the same in all parts of equal importance.

## Views

- **One projection.** Use one projection for all parts and objects in an image. Uniformity is more important than realism.
- **Depth from light.** In all views, depth comes mostly from light and contrast. Put strong contrast where two surfaces meet at a sharp angle.

### Three-quarter top-down

- The viewer is 45 degrees above the subject. About 3/4 of the top and 3/4 of the front are visible.
- Make the front faces shorter than in a front view. Keep the angles of the shapes, for example a roof, as in a front view.
- Put the light at a top corner.
- Draw a floor straight from above. Use the projection for the objects that stand on it.
- A character is a front view with more of the top of the head visible. Draw the down, up, and side views. Mirror the side view for the other side only if the character is symmetric.

### Three-quarter

- The viewer is 45 degrees from the center, and looks at a corner. About 3/4 of each side is visible.
- Keep the angles of the shapes, for example a roof, at 45 degrees as the sides get narrower.
- Put the light at a corner, so that the two sides have different values.

### Isometric

- **2:1 lines.** Make all edges on the ground plane 2:1 lines: 2 px across for each 1 px down. A true 30 degree line makes an uneven rhythm.
- **Vertical edges.** Make vertical edges straight vertical lines.
- **Lengths.** Make the edge lengths multiples of the 2 px step. Use even tile sizes.
- **Corners.** A sharp corner follows the 2:1 grid and tiles with no overlap. A wide corner looks softer, but the tiles must overlap. Use one corner style for all objects.
- **Circles.** For a circle on the ground plane, draw an isometric square. The circle touches the center of each side of the square.
- **Count.** The eye cannot judge isometric slopes. Count the pixels of each 2:1 step.
- **Faces.** Give each of the three visible faces a different value. Usually the top is the lightest, and the face away from the light is the darkest.

## Hardware limits

Use these limits for a strict retro look.

- **NES (`8-bit`).** A sprite palette has 3 colors and transparent. Each 8x8 tile of a sprite uses one palette, so each 8x8 tile has at most 3 colors, including the outline. A large character combines tiles with different palettes. A background has one palette for each 16x16 area.
- **Game Boy (`gameboy`).** The screen is 160x144 and has 4 shades. A sprite has 3 shades and transparent. Sprites are 8x8 or 8x16.

## Composition

- **Fill the canvas.** Make the subject large on the canvas, with a margin of at least 1 px.
- **Clear direction.** A three-quarter view or a slight tilt usually looks more alive than a straight front view.
- **No tangents.** Lines and edges that almost touch make the eye stop. Make them overlap clearly, or separate them clearly.

## Common defects

| Defect          | Description                                                     |
| --------------- | --------------------------------------------------------------- |
| Jaggy           | A sudden change of run length in a line or a curve.             |
| Double          | An extra pixel in the corner of a 1 px line.                    |
| Orphan          | A single pixel with no purpose.                                 |
| Banding         | Tone steps that align next to each other.                       |
| Pillow shading  | Shading from the outline to the center, with no light direction. |
| Mud             | Too many similar tones, with low contrast.                      |
| Blur            | Too much anti-aliasing.                                         |
| Noise           | Random pixels or random dithering.                              |
| Tangent         | Two edges that almost touch.                                    |
