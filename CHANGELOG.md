<a name="0.3.1"></a>
## 0.3.1 (2021-04-14)

Just a quick release because I broke docs.rs :)

#### Bug Fixes

* **build:**  remove .cargo/config.toml to make docs.rs happy ([393d9807](https://github.com/zkat/big-brain/commit/393d9807576d21c7234667b1f9914f1886579bd0))


<a name="0.3.0"></a>
## 0.3.0 (2021-04-14)

This is a major overhaul of the Bevy API. It removes `.ron` support
altogether, in favor of plain old Rust builders.

#### Breaking Changes

* The `.ron` Thinker definition API is gone. Use the ThinkerBuilder API instead.
* The `Action` and `Scorer` derive macros are gone, including all of `big_brain_derive`.
* Measures and Weights are gone.
* `big-brain` no longer exports everything from the toplevel. Use `big_brain::prelude::*` instead.

#### Bug Fixes

Probably.

#### Features

* New builder-based Thinker API!
* Composite Scorers: `FixedScore`, `AllOrNothing`, and `SumOfScorers`.
* Composite Action: `Steps`, for sequential Action execution.
