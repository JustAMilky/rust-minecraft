# Rust Minecraft
My silly attempt at a clone of minecraft written in Rust.

## Controls
`WSAD`: Move camera laterally

`Space/Left Shift`: Move camera up/down

`Arrow keys`: Turn camera

## Features
### Cubic chunks
This project uses cubic chunks that span all three axes of the world allowing for virtually infinite expansion in every direction


### World gen
3D density function-based world generation intended to emulate the style of post-1.18 Minecraft world gen.

### Fast, threaded chunk loading
Chunk generation jobs are managed by a separate thread, keeping the frame rate smooth while chunks are being processed. Chunk world gen and mesh gen jobs are completed by a team of worker threads to allow for fast generation and full resource utilisation.
