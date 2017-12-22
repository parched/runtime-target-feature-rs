# runtime-target-feature-rs
Rust procedural macro to enable target features at runtime

## How to use

You have to both add the derive-crate and the runtime to your dependencies:

```toml
[dependencies]
runtime-target-feature = { git = "https://github.com/parched/runtime-target-feature-rs" }
runtime-target-feature-rt = { git = "https://github.com/parched/runtime-target-feature-rs" }
```

Then, use the crate like this:

```rust
#![feature(proc_macro)]
#![feature(target_feature)]
#![feature(const_fn)]

extern crate runtime_target_feature;

use runtime_target_feature::runtime_target_feature;

#[runtime_target_feature("+avx2;+sse4.1")]
pub fn some_function() {
    // some code that would benefit from avx2 or sse4.1
}
```
This generates 3 versions of `some_function`, one with avx2, one with sse4.1 and the default that would have been generated without the attribute. When `some_function` is called it will correctly call the first version the current CPU supports, starting from the left. This means the binary code will get the added performance of avx2 if the CPU it's run on supports, otherwise will fallback to the usual version.

### Notes:
* The above isn't portable between archectures, so you should use `runtime_target_feature` with `cfg_attr` like
  ```rust
  #[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), runtime_target_feature("+avx"))]
  #[cfg_attr(target_arch = "arm", runtime_target_feature("+neon"))]
  pub fn some_function()
  ...
  ```
* The features come in sets for each version of the function so `runtime_target_feature("+a,+b;+c,+d")` will generate three versions, one with `a` and `b` enabled, one with `c` and `d` enabled, and the deafult.
