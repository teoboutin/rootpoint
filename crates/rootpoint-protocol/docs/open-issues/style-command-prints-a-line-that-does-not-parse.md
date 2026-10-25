---
kind: defect
---
# A style command built through the API with `style=` or `custom` runs, and prints a line that does not parse

## Summary

`Session::run` accepts a `Command::Style(StyleCommand::Add { .. })` or `StyleCommand::Set` whose
options hold `StyleOption::Named` or `StyleOption::Custom`: it ignores those options and replies
`ok`. The command prints as a line the grammar refuses, so a session holding it cannot be written
out as a script and run again.

## Details

### What

`parse_command` refuses `style add "X" custom` and `style set "X" style="Y"`, through
`value_options` of `path@rootpoint-protocol@src/command.rs`: "`custom` chooses a text's style, not a
value of a style". The session does not hold a command built through the API to the same rule:
`Session::style_values` of `path@rootpoint-protocol@src/session.rs` passes each option to
`apply_style`, which changes no value for `Named` and `Custom`, so the command runs.

Reproduction, a test of the protocol crate run against the branch that added the span commands:

```rust
let mut f = MemoryFiles::default();
let mut s = Session::new();
let c = Command::Style(StyleCommand::Add {
    name: "X".into(),
    options: vec![StyleOption::Custom],
});
let line = c.to_string();
// run: Ok(""); parse: Err(ParseError("`custom` chooses a text's style, not a value of a style"))
println!("run {:?} parse {:?}", s.run(c, &mut f), parse_command(&line));
```

The span style commands of the same branch refuse these options in the session,
`Session::span_values`, and `text_ops` already refuses a size the grammar refuses, for this reason.
No window action builds such a command today: `App::add_style_with` and `App::set_selected_style`
of `path@rootpoint-app@src/actions.rs` take their options from `StyleOption::values_of` and from
`App::style` of `path@rootpoint-app@src/panels.rs`, and neither builds `Named` or `Custom`.

### Why it matters

`design@rootpoint-protocol@protocol-transport` says a printed `Command` is a line that parses back
to it, so a session can be written out as a script and run again; a host that builds this command
breaks that, and the script of its session stops at that line.

### What would close it

`Session::style_values` refuses `StyleOption::Named` and `StyleOption::Custom` with the grammar's
reason, and a test of `path@rootpoint-protocol@tests/commands.rs` runs both commands through
`Session::run` and finds each refused with the document unchanged.
