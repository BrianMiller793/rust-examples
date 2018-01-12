# Rust Function Pointers and Structures

Rust function pointers can be fairly easy to use, but there are some tricks
and strange error messages when first learning to use them.  This example
details the process I went through to learn how to store them in a structure.

The example code is based on The Rust Programming Language, 2nd ed., section
12 and section 13.3, the minigrep program.  The answer comes from the
Rust Nomicon, section 3.7, Higher-Rank Trait Bounds (HRTBs).

At the point that I embarked on this escapade, I had completed section 12.
I've written lots of software in C#, C, Python, along with some Ruby and Java.
So I know what I want to do, and it's time to find out how it's done in Rust.

## Too Long, Don't Wanna Learn (i.e., Executive Summary)

    pub struct Config {
        ... snip ...
        pub search_fn: for<'r, 's> fn(&'r str, &'s str) -> Vec<&'s str>,
    }
    
    impl Config {
        pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
            args.next();
        ... snip ...
            let search_fn: for<'r, 's> fn(&'r str, &'s str) -> Vec<&'s str>;
            search_fn = if case_sensitive
                { search } else { search_case_insensitive };
            Ok(Config { query, filename, case_sensitive, search_fn })
    }
    
    pub fn run(config: Config) -> Result<(), Box<Error>> {
        ... snip ...
        for line in (config.search_fn)(&config.query, &contents) {

## Refactoring for Similar Signatures

The code in the example was acceptable, but one thing that stood out and
cried to be refactored was in the `run()` function.

    let results = if config.case_sensitive {
        search(&config.query, &contents)
    } else {
        search_case_insensitive(&config.query, &contents)
    };

Now, wouldn't that look nicer if it was a function pointer?  How about:

    let results = config.search_fn(&config.query, &contents);

or even,

    for line in config.search_fn(&config.query, &contents) {

and just drop that intermediate variable.

So first things first, let's put a function pointer in the `Config` structure.

### Function Pointers

I knew Rust had to support function pointers.  Given, right?  And of course
it does.  The tutorials even have a little project in section 19, Advanced
Features.  Excellent!  Let's see,

    fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {

so a function pointer is `fn(type) -> type`, so I just need to copy and
edit one of the `search()` functions. So I added `search_fn` to `Config`, so:

    fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {

becomes

    pub struct Config {
        pub search_fn: fn(&str, &'a str) -> Vec<&'a str>,

and `cargo build` tells me:

    minigrep $ cargo build
       Compiling minigrep v0.1.0 (file://.../minigrep)
    error[E0261]: use of undeclared lifetime name `'a`
      --> src/lib.rs:10:30
       |
    10 |     pub search_fn: fn(&str, &'a str) -> Vec<&'a str>,
       |                              ^^ undeclared lifetime
    
    error[E0261]: use of undeclared lifetime name `'a`
      --> src/lib.rs:10:46
       |
    10 |     pub search_fn: fn(&str, &'a str) -> Vec<&'a str>,
       |                                              ^^ undeclared lifetime

Hey, the errors tell me I've got stuff to do.  The function has lifetimes
specified for the arguments, so that must mean that the structure needs them,
too.  Throw some lifetime goodness on the structure, and it compiles.

    pub struct Config<'a> {
        pub search_fn: fn(&str, &'a str) -> Vec<&'a str>,

and that means that the implementation needs it, too:

    impl<'a> Config<'a> {
        pub fn new(mut args: std::env::Args) ->
            Result<Config<'a>, &'static str> {

Now the compiler tells me

    missing field `search_fn` in initializer of `Config<'_>`

Wow, Rust is really protecting me from myself!  Neat!  I just have to add
that, and I'm good to go.

    let search_fn: fn(&str, &'a str) -> Vec<&'a str>;
    search_fn = if case_sensitive
        { search } else { search_case_insensitive };

    Ok(Config { query, filename, case_sensitive, search_fn })

Yeah, that's it!  It compiles, and that means all I have to do is use it
in the `run()` function.  Now, how does Rust call a function pointer stored
in a structure?

### Using a Function Pointer stored in a Structure

Let's see, section 19.5 shows `f(arg) + f(arg)`, so that means this should
work:

    let results = config.search_fn(&config.query, &contents);

And the compiler replies:

    error[E0599]: no method named `search_fn` found for type `Config<'_>` in the current scope
      --> src/lib.rs:40:26
       |
    40 |     let results = config.search_fn(&config.query, &contents);
       |                          ^^^^^^^^^ field, not a method
       |
       = help: use `(config.search_fn)(...)` if you meant to call the function stored in the `search_fn` field

Compiler to the rescue! The real syntax is: `(config.search_fn)(...)`.  Fix that, and I'm good to go, right?

### The best-laid schemes o' mice an' men / Gang aft agley.

Um, no.  The compiler doesn't like me.  No love!  I got the error:

       Compiling minigrep v0.1.0 (file:///.../minigrep)
    error[E0597]: `contents` does not live long enough
      --> src/lib.rs:40:54
       |
    40 |     let results = (config.search_fn)(&config.query, &contents);
       |                                                      ^^^^^^^^ does not live long enough
    ...
    47 | }
       | - borrowed value only lives until here
       |
    note: borrowed value must be valid for the anonymous lifetime #1 defined on the function body at 35:1...
      --> src/lib.rs:35:1
       |
    35 | / pub fn run(config: Config) -> Result<(), Box<Error>> {
    36 | |     let mut f = File::open(config.filename)?;
    37 | |     let mut contents = String::new();
    38 | |     f.read_to_string(&mut contents)?;
    ...  |
    46 | |     Ok(())
    47 | | }
       | |_^
    
    error: aborting due to previous error
    
    error: Could not compile `minigrep`.

**WHAT?** But `contents` really only does need to live to the end of the
function! I don't need it to live after that.  Seriously, compiler, what gives?

### The Nomicon and HRTB

Doing a Google search on the error messages resulted in a reference to
The Nomicon.  That reference pointed me to the right direction, and it was
that and a compiler error that helped.

In my flailing to fix the problem, I managed to produce an error that
mentioned a type mismatch.  (I can't reproduce it right now, and the
following is an approximation.)

    expected for<'r> fn(&'r str, &str) -> Vec<&str>

I had danced around that error previously, hiding it away, but hiding an
error doesn't resolve it.  Well, in the Rust language appendix, here's
this definition for the `for` keyword:

    for - iterator loop, part of trait impl syntax, and higher-ranked lifetime syntax

Oh, lifetime syntax!  The error is about an anonymous lifetime, so the problem
must be about how I am specifying something.  What I needed to do was to
specify something similar to the `where` clause, and that's where Higher-Rank
Trait Bounds (HRTBs) are used.  With syntactic "sugar" the function looks
like this to the compiler:

    fn search<'a, 'b>(query: &'a str, contents: &'b str) -> Vec<&'b str> {

Rust's default lifetime syntax rules were confused by how I had specified
the lifetime.  Ignorance is blister!  So to fix the problem, all I needed
to do was change the member type:

    pub search_fn: for<'r, 's> fn(&'r str, &'s str) -> Vec<&'s str>,

The function has separate lifetimes for its arguments, and the return value
and the second parameter have the same lifetime.  Now everything compiles,
it passes the tests, and Rust loves me!

