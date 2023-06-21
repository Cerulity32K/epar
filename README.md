# Exclusively Polygons Alongside Rhythms
An event-based remake of Just Shapes and Beats' mechanics. Based on `soloud` for audio and `macroquad` for graphics.

# Challenge
- Be able to manage dynamic objects.
- Properly synchronize audio with events.
- Be able to create and debug collisions using math.

# Findings
- All dynamic objects are `Box`es, and a `box_clone(&self) -> Box<dyn ...>` trait method keeps dynamic resolution (`dyn Clone` is not resolvable as `Self` can vary).
- Soloud allows you to query sound playback times, which are used to trigger events and update projectiles.
- Employing a collision sweep across the screen with squares can give a rough shape of collision boundaries, allowing for better debugging.
