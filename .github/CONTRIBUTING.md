<!-- omit in toc -->

# Contributing to Astrograph

First off, thanks for taking the time to contribute! ❤️

All types of contributions are encouraged and valued. See the [Table of Contents](#table-of-contents) for different ways to help and details about how this project handles them. Please make sure to read the relevant section before making your contribution. It will make it a lot easier for us maintainers and smooth out the experience for all involved. The community looks forward to your contributions. 🎉

> And if you like the project, but just don't have time to contribute, that's fine.
> There are other easy ways to support the project and show your appreciation, which
> we would also be very happy about:
>
> - Star the project
> - Skeet/Tweet/Blog about it
> - Refer this project in your project's readme
> - Mention the project at local meet-ups and tell your friends/colleagues

<!-- omit in toc -->

## Table of Contents

- [I Have a Question](#i-have-a-question)
  - [I Want To Contribute](#i-want-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Your First Code Contribution](#your-first-code-contribution)
- [Style guides](#style-guides)
  - [Commit Messages](#commit-messages)
  <!-- TODO:
- [Join The Project Team] (#join-the-project-team)
  -->

## I Have a Question

> If you want to ask a question, we assume that you have read the available [Documentation](https://docs.rs/astrograph/latest/astrograph/).

Before you ask a question, it is best to search for existing [Issues](https://github.com/2sugarcubes/astrograph/issues)
that might help you. In case you have found a suitable issue and still need clarification,
you can write your question in this issue as a comment. <!-- Maybe if it's more popular
It is also advisable to search the internet for answers first.
-->

If you then still feel the need to ask a question and need clarification, we recommend
the following:

- Open an [Issue](https://github.com/2sugarcubes/astrograph/issues/new?template=question)
  with the question template.
- Provide as much context as you can about what you're running into.
- Provide project and platform versions (OS, rustc, cargo, etc), depending on
  what seems relevant.

We will then take care of the issue as soon as possible.

<!--
You might want to create a separate issue tag for questions and include it in this
description. People should then tag their issues accordingly.

Depending on how large the project is, you may want to outsource the questioning,
e.g. to Stack Overflow or Gitter. You may add additional contact and information
possibilities:
- IRC
- Slack
- Gitter
- Stack Overflow tag
- Blog
- FAQ
- Roadmap
- E-Mail List
- Forum
-->

## I Want To Contribute

> ### Legal Notice <!-- omit in toc -->
>
> When contributing to this project, you must agree that you have authored 100%
> of the content, that you have the necessary rights to the content and that the
> content you contribute may be provided under the project licence --- the [GNU
> Lesser General Public License V3.0](https://www.gnu.org/licenses/lgpl-3.0.html#license-text)
>
> [summary](https://choosealicense.com/licenses/lgpl-3.0/)
> i.e. you expressly grant any patent rights related to the source code you provide.

### Reporting Bugs

- Open an [Issue with the bug report template](https://github.com/2sugarcubes/astrograph/issues/new?template=bug_report.md).
- Provide as much context as you can about what you're running into, e.g. input,
  expected output, and problem lines if you can identify them in the format
  `file/path.rs:START LINE NUMBER-END LINE NUMBER` or `file/path.rs:FUNCTION NAME`.
- Provide project and platform versions (OS, rustc, cargo, etc), depending on
  what seems relevant.

<!-- omit in toc -->

#### Before Submitting a Bug Report

A good bug report shouldn't leave others needing to chase you up for more information.
Therefore, we ask you to investigate carefully, collect information and describe
the issue in detail in your report. Please complete the following steps in advance
to help us fix any potential bug as fast as possible.

- Make sure that you are using the [latest version](https://crates.io/crates/astrograph/versions).
- Determine if your bug is really a bug and not an error on your side e.g. using
  incompatible environment components/versions (Make sure that you have read the
  [documentation](https://docs.rs/astrograph/latest/astrograph/). If you are looking
  for support, you might want to check [this section](#i-have-a-question)).
- To see if other users have experienced (and potentially already solved) the same
  issue you are having, check if there is not already a bug report existing for your
  bug or error in the [bug tracker](https://github.com/2sugarcubes/astrograph/issues?q=label%3Abug).
- Also make sure to search the internet (including Stack Overflow) to see if users
  outside of the GitHub community have discussed the issue.
- Collect information about the bug:
  - Stack trace (Traceback) (putting a panic in the source code is an easy way
    of generating this)
  - OS, Platform and Version (Windows, Linux, macOS, x86, ARM)
  - Version of the rustc, and cargo --- depending on what seems relevant.
  - Possibly your input and the output
  - Can you reliably reproduce the issue? And can you also reproduce it with older
    versions?

<!-- omit in toc -->

#### How Do I Submit a Good Bug Report?

> You must never report security related issues, vulnerabilities or bugs including
> sensitive information to the issue tracker, or elsewhere in public. Instead sensitive
> bugs must be sent by email to <lukemagnusson@live.com>.

<!-- You may add a PGP key to allow the messages to be sent encrypted as well. -->

We use GitHub issues to track bugs and errors. If you run into an issue with the
project:

- Open an [Issue with the bug report template](https://github.com/2sugarcubes/astrograph/issues/new?template=bug_report.md).
- Explain the behavior you would expect and the actual behavior.
- Please provide as much context as possible and describe the _reproduction steps_
  that someone else can follow to recreate the issue on their own. This usually includes
  your code. For good bug reports you should isolate the problem and create a reduced
  test case.
- Provide the information you collected in the previous section.

Once it's filed:

- The project team will label the issue accordingly.
- A team member will try to reproduce the issue with your provided steps. If there
  are no reproduction steps or no obvious way to reproduce the issue, the team will
  ask you for those steps and mark the issue as `needs-reproduction`. Bugs with the
  `needs-reproduction` tag will not be addressed until they are reproduced.
- If the team is able to reproduce the issue, it will be marked `needs-fix`, as well
  as possibly other tags (such as `critical`), and the issue will be left to be
  [implemented by someone](#your-first-code-contribution).

<!--
You might want to create an issue template for bugs and errors that can be used
as a guide and that defines the structure of the information to be included. If
you do so, reference it here in the description.
-->

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for Astrograph,
**including completely new features and minor improvements to existing functionality**.
Following these guidelines will help maintainers and the community to understand
your suggestion and find related suggestions.

<!-- omit in toc -->

#### Before Submitting an Enhancement

- Make sure that you are using the latest version.
- Read the [documentation](https://docs.rs/astrograph/latest/astrograph/) carefully
  and find out if the functionality is already covered, maybe by an individual
  configuration.
- Perform a [search](https://github.com/2sugarcubes/astrograph/issues) to see if
  the enhancement has already been suggested. If it has, add a comment to the existing
  issue instead of opening a new one.
- Find out whether your idea fits with the scope and aims of the project. It's up
  to you to make a strong case to convince the project's developers of the merits
  of this feature. Keep in mind that we want features that will be useful to the
  majority of our users and not just a small subset. If you're just targeting a
  minority of users, consider writing an add-on/plugin library.

<!-- omit in toc -->

#### How Do I Submit a Good Enhancement Suggestion?

Enhancement suggestions are tracked as [GitHub issues](https://github.com/2sugarcubes/astrograph/issues?q=is%3Aissue%20label%3Aenhancement%20).
You should also use the [feature request template](https://github.com/2sugarcubes/astrograph/issues/new?q=is%3Aissue+label%3Aenhancement+&template=feature_request.md).

- Use a **clear and descriptive title** for the issue to identify the suggestion.
- Provide a **step-by-step description of the suggested enhancement** in as many
  details as possible.
- **Describe the current behavior** and **explain which behavior you expected to
  see instead** and why. At this point you can also tell which alternatives do not
  work for you.
- **Explain why this enhancement would be useful** to most Astrograph users.
<!-- You might want to create an issue template for enhancement suggestions that
can be used as a guide and that defines the structure of the information to be
included. If you do so, reference it here in the description. -->

### Your First Code Contribution

<!-- TODO
include Setup of env, IDE and typical getting started instructions?
-->

1. Look for issues with the [first-timers-only](https://github.com/2sugarcubes/astrograph/issues?q=is%3Aissue%20state%3Aopen%20label%3Afirst-timers-only%20no%3Aassignee%20)
   (if this is your first time contributing anywhere) or [good-first-issue](https://github.com/2sugarcubes/astrograph/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22good%20first%20issue%22%20no%3Aassignee%20)
   (if this is your first time contributing to the repository) labels.
   - These issues have been chosen because they are low commitment --- they should
     only require modifying one function/documentation block (and possibly the
     relevant `Cargo.toml`) --- very detailed requirements, and in the case of
     `first-timers-only` will provide tested step by step instructions for implementing
     the changes so you can focus on the steps around creating a pull request (PR).
1. Fork the repository
   1. Go to the [homepage of the repository](https://github.com/2sugarcubes/astrograph)
   1. Click [fork](https://github.com/2sugarcubes/astrograph/fork) in the top right
      corner.
   1. You will find a new repo in _your_ repositories, it will be located at
      `https://github.com/[YOUR_USER_NAME]/astrograph`
   1. clone your fork of Astrograph, in the command line run `git clone git@github.com:[YOUR_USER_NAME]/astrograph`
1. Create a branch
   - it is recommended you name your branch like the following `git switch -c (feat|fix|docs|test|...)/
[feature or bug this branch will provide]`, e.g. `git switch -c docs/add-documentation-to-foo`
   - For more information on naming your branch you can reference [this article](https://medium.com/@abhay.pixolo/naming-conventions-for-git-branches-a-cheatsheet-8549feca2534)
   - This naming convention is only a suggestion, your PR **will not** be rejected
     if you do not use it, it simply makes building changelogs easier.
1. Create your changes
1. Commit your changes
   - It is recommended you use [conventional commit messages](https://www.conventionalcommits.org/)
   - e.g. `feat(binary)!: breaking feature added to the binary, e.g. adding a
required flag`
   - `fix(body): non-breaking bug-fix to the body struct/file`
1. Create a pull request for your changes
   1. Go to your fork of the repository `https://github.com/[YOUR_USER_NAME]/astrograph`
   2. Click on `Compare & pull request` or visit this link:
      `https://github.com/2sugarcubes/astrograph/pull/new/
[your branch name, with any forward slashs]`

## Style-guides

This repository uses clippy with the pedantic group set to deny in continuous integration
(CI) i.e. `cargo clippy -- -Wclippy::pedantic` will warn you about anything that
would get denied in CI, it is recommended to toss in some [allow statements](https://doc.rust-lang.org/stable/clippy/usage.html#lint-configuration)
if the clippy suggestion seems more detrimental than helpful (e.g. if it suggests
adding a panic section to documentation even if it can only panic in a `cfg!(test)`
block) with a comment as to why it was allowed.
It is also recommended that you run `cargo fmt` and `typos --locale en-us` (or
your preferred spellchecker that is configured to use US English) to format your
rust code according to the rust formatting guide, and catch any spelling issues
before you push.

### Commit Messages

It is recommended but not required that you use [conventional commit messages](https://www.conventionalcommits.org/),
this is to make generating changelogs much quicker and less likely to miss changes.
If you do not use conventional commits in your changes the merge commit will be
made with a conventional commit that summarises your changes.

> Examples of conventional commit messages:
>
> `fix(wasm tests): skip coverage until wasm-pack coverage is stable`
>
> `test: increased code coverage by 5%`
>
> `feat!: added constelations to observatories`

<!-- TODO:
## Join The Project Team

 -->

<!-- omit in toc -->

## Attribution

This guide is based on [contributing.md](https://contributing.md/generator)!
I would recommend it after some modifications for your repository.
