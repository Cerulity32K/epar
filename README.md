# Exclusively Polygons Alongside Rhythms
An event-based remake of Just Shapes and Beats' mechanics. Based on `soloud` for audio and `macroquad` for graphics. Made to learn more advanced dynamic Rust.

# Usage
In the main menu, a list of levels will appear.\
To play the game, simply click on one of the levels. You will be sent to the level.\
Use WASD to move, and space to dash, which speeds you up and makes you invincible for a short period of time.\
Currently, lives & death are not implemented, and do not affect you.\
You can hold U in the main menu to view and "play" levels under development.

# Challenge
- Be able to manage dynamic objects.
- Properly synchronize audio with events.
- Be able to create and debug collisions using math.

# Findings
- All dynamic objects are `Box`es, and a `box_clone(&self) -> Box<dyn ...>` trait method keeps dynamic resolution (`dyn Clone` is not resolvable).
- Soloud allows you to query sound playback times, which are used to trigger events and update projectiles.
- Employing a collision sweep across the screen displayed with squares can give a rough shape of collision boundaries, allowing for better collision debugging.
