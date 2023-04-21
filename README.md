# script-macro

An **experimental** way to write simple proc-macros inline with other source code.

Did you ever end up getting frustrated at the boilerplate involved in writing
proc macros, and wished you could just write a Python or Bash script to
generate the code instead?

```rust
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[script_macro::run_script_on(r##"
        let output = item;

        for x in 0..10 {
            for y in 0..10 {
                output += `
                #[test]
                fn it_works_${x}_${y}() {
                    it_works(${x}, ${y}, ${x + y});
                }`;
            }
        }

        return output;
    "##)]
    fn it_works(x: usize, y: usize, out: usize) {
        assert_eq!(add(x, y), out);
    }
}
```

Macros are not Rust source code, instead they are written in the [RHAI](https://rhai.rs/) scripting language. This comes with advantages and disadvantages:

* **Downside: No access to the Rust crate ecosystem** -- RHAI is its own entire
  separate language, and therefore you can't use Rust crates inside. RHAI _can_
  be extended with custom Rust functions, but `script-macro` does not support
  that yet. For now, `script-macro` exposes a few helpers commonly useful in
  code generation.

* **Upside: Sandboxability** -- Proc macros executed with `script-macro` cannot
  access the internet or perform arbitrary syscalls. Proc macros _are_ given
  full access to the filesystem via the functions available through
  [`rhai-fs`](https://docs.rs/rhai-fs/latest/rhai_fs/), but in a future version
  this could be configurable, for example read-only access or restricted to
  certain directories.

* **Downside: Dependency on RHAI runtime** -- RHAI is an entire language
  runtime that has to be compiled _once_ before any of your proc macros run.

* **Upside: No recompilation when editing proc macros.** -- Proc macros are
  interpreted scripts. When editing them, only the containing crate needs to be
  recompiled, not `script-macro` itself. This _could_ end up being faster when
  dealing with a lot of proc macros.

  See also [watt](https://github.com/dtolnay/watt), which appears to have
  similar tradeoffs about compilation speed (compile runtime for all macros
  once, run all macros without compilation)

## API

There are two main macros to choose from:

* `script_macro::run_script_on` -- Attribute macro that executes a given script
  with the annotated function/module's sourcecode available as a global string
  under `item`.

  The return value of the script is the source code that the item will be
  replaced with.

  Here is a simple script macro that adds `#[test]` to the annotated function.

  ```rust
  #[script_macro::run_script_on(r##"
      return "#[test]" + item;
  "##)]
  fn it_works(x: usize, y: usize, out: usize) {
      assert_eq!(add(x, y), out);
  }
  ```

* `script_macro::run_script` -- Function macro that executes the given script. There are no inputs.

  ```rust
  script_macro::run_script!(r##"
      return `fn main() { println!("hello world"); }`;
  "##);
  ```

## Script API

From within the script, the entire stdlib of RHAI + the functions in
[`rhai-fs`](https://docs.rs/rhai-fs/latest/rhai_fs/) are available.

Additionally, the following functions are defined:


* `parse_yaml(String) -> Dynamic` -- Takes YAML payload as string and returns
  the parsed payload as unstructured data (such as, RHAI object map or array).

* `parse_json(String) -> Dynamic` --  Takes JSON payload as string and returns
  the parsed payload as unstructured data (such as, RHAI object map or array).

* `stringify_yaml(Dynamic) -> String` -- Convert a RHAI object to a YAML
  string, inverse of `parse_yaml`.

* `stringify_json(Dynamic) -> String` -- Convert a RHAI object to a YAML
  string, inverse of `parse_json`.

* `glob(String) -> Vec<PathBuf>` -- Takes a glob pattern and returns a list of paths that match it.

* `basename(PathBuf) -> String` -- Returns the `.file_name()` of the given
  path, or the entire path if there is none.


## Examples

Check out the [example crates](./example_crates) to see all of the above in action.

## License

Licensed under the MIT, see [`./LICENSE`](./LICENSE).
