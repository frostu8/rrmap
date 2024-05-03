# Zen Map Tool
**Zen Map Tool** is a keyboard-based modal approach to creating courses in
[Dr Robotnik's Ring Racers][1]. It uses [Rust][2] to stay *blazingly fast*, and
[`bevy`][3] for easy maintainability and access (if you like ECS that is).

This isn't meant to be a replacement for **Ultimate Zone Builder**, but instead
as an alternative for those who are willing to challenge the intricacies of
this fresh editor. The ~~documentation~~ (soon to be, this is a wishlist right
now) intends to be a fresh go-over of creating maps in **Zen Map Tool**,
including basic knowledge of UDMF mapping and how to make complex architecture,
while also detailing common workflows and usage of tools.

## Features
* **Keyboard-based Modal Editing**  
  Zen Map Tool's design philosophy is to keep your hand off the mouse as much
  as possible. Use Blender-like keybinds to manipulate your map and change
  editing contexts. The heightened control of the mouse can still be used to
  manipulate complex map constructions, and is encouraged when it is more
  convenient!
* **Ultimate Control**  
  Zen Map Tool does not expose everything by default. Schemas and
  configurations control how the user is allowed to interact with the resulting
  UDMF file. However, this is a totally *opt-out* experience, which allows
  those who are more familiar with low-level technology to both play around
  with the format and once again feel the gentle embrace of a visual editor.
* **Ring Racers-first**  
  The default config of the editor is meant to work for Ring Racers maps out of
  the box. An ACS editor is also built in to welcome ACS.
* **Blazingly Fast I/O**  
  Zen Map Tool's UDMF parser and printer is as fast as Ring Racer's internal
  hot parser written in C++, so you spend less time waiting for loads.
  (benchmarks pending...)

## FAQ
**Q: Why not "Zen Builder?"**

That was my first name, but the acronym "ZB" may make it easy to confuse with
"Zone Builder," an existing editor for SRB2.

[1]: https://kartkrew.org/
[2]: https://www.rust-lang.org/
[3]: https://bevyengine.org/
