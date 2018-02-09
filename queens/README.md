# N-Queens Problem

## History

In 1848, Max Bezzel published a puzzle for chess: how can eight queens be
placed on a chess board such that none can attack another queen?

Since then, there have been many solutions.  This is a problem I had fun
solving when I was in high school, on a 4MHz CP/M machine, in interpreted
BASIC.

## Algorithm

Instead of using backtracking like I first did, I decided to use constraints.
Each row on a board is tested for a safe column for the queen.  The queen
is placed, and then the queen's attack vectors are marked, thus forming the
constraints.  The board is placed on a stack.  So for an empty board, eight
new boards are placed on the stack.  Then the fun begins: popping a board
off the stack, trying to place a queen, and then pushing a board with a new
queen on the stack.  Repeat until the stack is empty.

So the next board from the stack will start on row 2, with two columns marked
as "reserved," leaving six columns where queens can be placed.  Six new boards
are placed back on the stack.

### Performance

On my little laptop, for single-threaded performance,
`time cargo run --release | wc -l` finished a 12x12 board in 0.57s,
yielding the expected 14,200 solutions, and a 14x14 board in 13.9s, for
365,596 solutions.

## Rust Neophyte

Yes, I'm new to the Rust language.  I finished the Rust language tutorial
not too long ago.  My main languages have been C, assembly, and C#.  A litle
Java here and there, but not much.  So last year, during some idle time, I
decided to learn Rust.  

Where to go with this?  The design lends it to multithreading. I also plan to
make a little cloud application out of this, distributing it among my
various Pi's and Pi-like boards.
