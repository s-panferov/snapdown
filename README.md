# SnapDown 

This library is a little helper to write Markdown-based expectation tests.

Those kind of tests are mostly useful for something like parsers, when you have some texual input and text-serializable output (like an AST tree).

The main convenience of this library is to have both *inputs* and *outputs* in the same file. 

Works best with `datatest`.

## Example

Test code example:

```rust
fn parser(path: &Path) -> datatest_stable::Result<()> {
	run_test::<(), _>(path, |blocks| {
		blocks.chunks_mut(2).for_each(|test| {
			let input = &test[0];
			let output = &test[1];
			
			let syntax = parse(input));
			output.result.set(Some(format!("{:#?}", syntax)));
		});
	})
}

datatest_stable::harness!(parser, "tests/parser", r".*");
```

Markdown test file example:

````
```markdown
<Link href="/documents/test/a" />
```

```rust
Root [
    Jsx { closed: true } [
        AngleStart,
        Text ["Link"],
        Space,
        JsxAttr [
            Text ["href"],
            Equals,
            JsxValue [
                String [
                    Quote,
                    ForwardSlash,
                    Text ["documents"],
                    ForwardSlash,
                    Text ["test"],
                    ForwardSlash,
                    Text ["a"],
                    Quote,
                ],
            ],
        ],
        Space,
        ForwardSlash,
        AngleEnd,
    ],
]
```
````

## Configuration

### CLI parsing

`run_test`'s first generic argument allows you to provide a Serde deserializable type to control accepted CLI arguments.


### Drafts

Test output annotated with `--draft` will re-generate without diff errors. This is useful for quick iterations while you work on something:

````
```rust --draft
/* this will re-generate */
```
````

Use `SNAPDOWN_REFRESH=1` environment variable to re-generate all the outputs.

