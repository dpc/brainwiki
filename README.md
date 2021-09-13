<!-- README.md is auto-generated from README.tpl with `cargo readme` -->

<p align="center">
  <a href="https://travis-ci.org/dpc/brainwiki">
      <img src="https://img.shields.io/travis/dpc/brainwiki/master.svg?style=flat-square" alt="Travis CI Build Status">
  </a>
  <a href="https://crates.io/crates/brainwiki">
      <img src="http://meritbadge.herokuapp.com/brainwiki?style=flat-square" alt="crates.io">
  </a>
  <a href="https://gitter.im/dpc/dpc">
      <img src="https://img.shields.io/badge/GITTER-join%20chat-green.svg?style=flat-square" alt="Gitter Chat">
  </a>
  <br>
</p>

# brainwiki

**NOTE**: 
I rewrote the whole thing as [tagwiki](https://github.com/dpc/tagwiki)
which is more complete than this project.

See [wiki](https://github.com/dpc/brainwiki/wiki) for current project status.

BrainWiki is a wiki where everything is addressed using tags. This allows
organization without any premeditated structure.

Created for personal use, to help me gather and view my messy md
notes.

Eg.

```markdown
[My idea about brainwiki](/idea/brainwiki)
```

will link to any/all pages that contain #idea and #brainwiki tags,
potentially broadening the search to the first best match.

Goals:

* minimalism and simplicity
* easy deployment
* low resource consumption

Current status: usable in it's basic form.

It supports:

* markdown
* watching for FS changes

In plans:

* ACE editor integration

UI based on Bootstrap. Written in Rust using actix-web.

#### Using

Clone, `cargo build`. Run the binary with `--data <dir-with-md-files>`

# License

brainwiki is licensed under: MPL-2.0
