# Hangry River Horse [![Build Status](https://travis-ci.org/excaliburHisSheath/hangry-river-horse.svg?branch=master)](https://travis-ci.org/excaliburHisSheath/hangry-river-horse)

---

> Go to http://hungryhipp.us/ to see the current version in action!

Hangry River Horse is a competitive multiplayer game inspired by the classic
Hasbro game Hungry Hippos. It is being developed for the Scientific Games
Artists Among Us showcase.

## Contributing

HRH is built using [Rust]+[Rocket] on the backend, and JavaScript+[Phaser] on
the front end. In order to work on HRH you'll need to install Rust.

To install Rust, use [Rustup]. Rustup is the official tool for installing Rust,
updating your installation, and managing having multiple versions of Rust
installed. Follow the instructions on the [Rustup website](https://rustup.rs/)
to install. When prompted for which toolchain you'd like to use, select
`nightly`. For all other prompts you can use the default options.

> NOTE: If you installed Rustup and didn't specify `nightly` as the default
> toolchain, you can run `rustup default nightly` from the terminal to set
> nightly as the default.
>
> We need to use the nightly toolchain for HRH because Rocket makes use of
> powerful-but-unstable features in the Rust compiler to make it very easy
> to write a web service.

With Rust installed, running the project locally is as simple as running
`cargo run` in the terminal within the `hangry-river-horse` repo. Cargo, Rust's
package manager and build tool, will automatically install any dependencies and
then compile the current version of the project. Once the build completes,
you should see Rocket's debug output:

![cargo-build](https://user-images.githubusercontent.com/1900829/27160737-fafc8404-513b-11e7-9cd4-1f8f50fb514f.PNG)

Once the server is running, you should be able to navigate to
http://localhost:8000/ and see the app running live on your machine. You should
Also be able to see Rocket log information web requests as you interact with
the app in your browser.

### Working on the Backend

The code for the backend is in the `src/` directory. Once you've made changes
that you'd like to test, kill the currently running server with `ctrl+C`, then
run `cargo run` again. Once your changes compile and the server is running, you
can refresh http://localhost:8000 to see the results of your changes.

Code style should conform to the [Rust Style Guide]. You can also check out
the [official code example] which demonstrates what code should look like in a
variety of cases.

### Working on the Frontend

Frontend work is even easier than backend work: Once you've made your changes,
simply refresh the page in your browser and you should see your changes. The
frontend files are served statically out of the `www/` directory, so any changes
to frontend files should appear the next time your browser asks for them.

> NOTE: If you refresh your browser page and don't see your changes, make sure
> your browser isn't caching the old version of the files. The server isn't
> configured to cache the files, so if your changes don't appear, then it's
> likely your browser's cache needs to be flushed. In Chrome, there's an option
> to disable caching while the debug tools are open.

### Build and Deployment Automation

HRH is [setup with Travis] to automatically build any commits that are pushed on
any branch. Pull requests are also built automatically, and cannot be merged
until the build passes. Once changes are merged into `master`, Travis will
automatically deploy the new version of the app to http://hungryhipp.us/, no
need to do any manual deployment work!

[Rust]: https://www.rust-lang.org/
[Rustup]: https://rustup.rs/
[Rocket]: https://rocket.rs/
[Phaser]: http://phaser.io/
[setup with Travis]: https://travis-ci.org/excaliburHisSheath/hangry-river-horse
[Rust Style Guide]: https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/guide/guide.md
[official code example]: https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/example/lists.rs
