# agb

A GameBoy emulator written in rust.

TODO:

    1. dmg ppu timings
    2. graphical debugger
    3. test what happens to lcd stat when you disable the lcd on a dmg (what mode does lcd stat report etc)
    4. fix oam dma (don't do it all at once)
    5. fix some graphical bugs in dmg mode
    6. add sound
    7. cgb double speed mode
    8. cgb ppu implementation

BUGS:

    1. Pokemon Red
        a. Sprites on edge of screen disappear
        b. Some sprites are transparent when they shouldn't be (ex: characters in pf oaks lab)
