# Dining Philosophers

Source: Communicating Sequential Processes, by C. A. R. Hoare, May 18, 2015

## Example: The Dining Philosophers, p. 55

For Hoare's example of concurrency, a group shares a dining table.  Each
place at the table consists of one plate, and a utensil on either side of the
plate, which is shared with the neighboring table place.

All of the diners are seated at once, and attempt to acquire utensils to eat.
Let us supposed that they all grab the utensil on the right, and then the
left.  But wait, everybody has a utensil, and none are left over!  Nobody can
eat, so everybody starves.  The solution is that one of the philosophers grabs
the utensil on the left first. Everybody eats.

## Solving the Word Problem

The word problem posed by Hoare is that philosophers enter the dining room,
seat, eat, and then leave.  And what if there are more philosophers than
places at the dining table?

The common solution involves a static situation, where a deadlock problem is
shown to be solved.  However, the real world is more dynamic than that.

### Initial Solution

Without really looking a the standard mutex implementation, it seemed that
it would be good to have a set of seats and diners.  Get a fork, and then
release a fork.  There's a few different ways to solve the problem when the
mutex can be programatically released.

Unfortunately, somebody decided that the standard mutex is only released when
the LockResult goes out of scope.  Yeah, let's figure out what that means.

That means that what you have is a tar baby, and you is Brer Rabbit.  You are
stuck to that LockResult for the life of your function, and you can't acquire
and release it on a whim.  For solving the problem, this makes some of the
possible solutions very messy.  So some alternate solutions were dumped.

The solution was to use a thread pool with one thread for each place at the
table.  The philosophers were placed in a queue.  When a philosopher is seated
at the table, first an attempt is made to find a place with two forks.  Failing
that, any place will do.

Can deadlocks occur?  Yes!  Since the mutex release in the standard crate is
not a fair release, a right-hand deadlock can occur.  Ok, we know how to deal
with that by making one setting left-handed instead of right-handed.  Can a
deadlock still occur?

Yes, the standard crate mutex can still deadlock.  This is not seen when there
are only a relative handful of philosophers.  But when there are a many
iterations, the deadlock occurs like this: A has fork.  B waits for fork.  A
puts fork down, and C picks up fork.  C puts down fork.  D waits for fork.
But neither B nor D pick up the fork!  (You can see this in the StdCrate
branch.)  This doesn't happen all the time, but when a sequence of runs is
performed, it's evident.

### Using Parking Lot

The parking\_lot crate has a different mutex algorithm, along with programmatic
release of the mutex.  After switching to this design, there were no more
deadlocks.

