# N-Queens Problem

## History

In 1848, Max Bezzel published a puzzle for chess: how can eight queens be
placed on a chess board such that none can attack another queen?

Since then, there have been many solutions.  The first was published in 1850
by Franz Nauck.  This is a problem I had fun solving when I was in high
school, on a 4MHz CP/M machine, in interpreted BASIC.

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
365,596 solutions. (And a 15x15 board produces 2,279,184 in 104 seconds.)

## Multithreading

The first step was to split `main.rs` into the queens library and a method to
test a board.  Since an end goal is to farm the work out in a cloud,
`test_set_board()` keeps its own stack of boards instead of one stack being
shared amongs all the threads.  A test showed no performance degradation for
a single thread.

Once that was complete, the next step was to integrate the thread pool library
from the `webserver` tutorial project.  Cargo.toml was modified, and
`test_set_board()` was wrapped by `pool.execute()`.  A test showed essentially
no performance degradation for a signle thread. However, a thread pool with
two threads showed an increase from 104 seconds to 113 seconds!

### Refactoring the UI

#### Find the Bottleneck

OK, so a `println!()` isn't much of a user interface, but it is there, and it
was what was messing up performance.  Without printing the board solutions, the
time for a 15x15 board became 19.6s for one thread, 10.2s for two threads, 9.3s
for three threads, and 8.9s for four threads.

Hey, shouldn't the peformance be getting much better with more threads?  No!
Only real cores really count!  My laptop has an Intel Core i7 processor,
which has two cores and hyperthreading.  The real cores get engaged for
the first two threads, and then the hyperthreading doesn't really add
anything significant.  Hyperthreading is hyperbole.

#### Woah, mule, I need that count!

As written, the webserver's threading library is exiting by the time we want
the results.  Well, it would be nice to wait for the threads, and get the
end result before exiting.

The threads are joined in the destructor `Done`,
so that needs to be changed.  Modify `Done`, and move the internals
to a new public function, `wait()`.  In the new function, clear the `workers`
after its done to ensure that the `join()` won't happen twice.  `wait()` is
called before the solutions are printed, so the main thread waits until
the thread pool is done, and then prints the total.

#### Saving the Results

Well, what good is it to do a bunch of work, and then get no data?  Instead
of printing a board when it was completed, the best strategy was to update a
collection of solutions.  A collection of boards is wrapped by a mutex,
wrapped by an atomic reference count, and then a clone of that atomic
reference count is passed to `test_set_board()`.  This is based on the Rust
Language book, second edition, Shared State example.

Saving the results to a vector gives a time of 11.2s for a 15x15 board and
two threads.  Using a counter, like in the Shared State example, results
in 10.3s.

##### Mutexes and Operations

This example wraps an atomic reference count and mutex to protect an object.
While this is necessary for a complex object, this isn't necessary for a
simple integer, for a simple operation like increment or decrement.  The
actual processor instruction, INC or DEC, is atomic.  This means that the
hardware has already handled everything for the software, even in a
multi-processor system.

## Rust Neophyte

Yes, I'm new to the Rust language.  I finished the Rust language tutorial
not too long ago.  My main languages have been C, assembly, and C#.  A litle
Java here and there, but not much.  So last year, during some idle time, I
decided to learn Rust.  

Where to go with this?  The design lends it to multithreading. I also plan to
make a little cloud application out of this, distributing it among my
various Pi's and Pi-like boards.
