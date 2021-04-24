<a name="0.3.5"></a>
## 0.3.5 (2021-04-24)

Previously, if a `Picker` re-picked the same action, and that action had been
set to `Success` or `Failure`, it would just keep running the action in that
state until it was time to switch to a different one.

With this version, that behavior is changed, and `Failure` and `Success`
actions that are re-picked will be respawned entirely (not even reused).

Cheers to `@doomy` on Discord for pointing out this weird behavior!

#### Bug Fixes

* **thinker:**  launch a new action when the current action is in an end state ([80d23f2f](https://github.com/zkat/big-brain/commit/80d23f2f2337a863c9cc3afbf944b25e3911db8c))


<a name="0.3.4"></a>
## 0.3.4 (2021-04-23)

Welp. Turns out Thinkers were completely broken? This should work better :)

#### Bug Fixes

* **prelude:**  Export ThinkerBuilder from prelude ([06cc03e1](https://github.com/zkat/big-brain/commit/06cc03e1dd563c708bff276f7a194c8c81a00a5a))
* **thinker:**
  *  disposed of ActiveThinker and circular state-setting ([7f8ed12b](https://github.com/zkat/big-brain/commit/7f8ed12b112152c3f8d548d0a2208cefdb1581af))
  *  Need to do proper ptr_eq comparison here ([037a7c0d](https://github.com/zkat/big-brain/commit/037a7c0d0da065ea4cb5642047302d6bda13c670))


<a name="0.3.3"></a>
## 0.3.3 (2021-04-18)

This fixes an issue with more children being added to an Actor causing Thinkers to get clobbered in really annoying ways.

#### Bug Fixes

* **thinker:**  stop using the child/parent system for toplevel thinkers ([db16e2f6](https://github.com/zkat/big-brain/commit/db16e2f6ee97777b4df12e4ae435bf27b8012c7c))


<a name="0.3.2"></a>
## 0.3.2 (2021-04-17)

This is a quick bugfix. Shoutout to [@ndarilek](https://github.com/ndarilek)
for finding this one and giving me a chance to debug it!

tl;dr: Bevy's hierarchy system *requires* that all children have `Transform`
and `GlobalTransform` also attached, otherwise it just... kills them.

#### Bug Fixes

* **scorer:**  stop attaching duplicate scorers ([10a6d022](https://github.com/zkat/big-brain/commit/10a6d022ec682e33b98309318020c9068be4cea2))
* **thinker:**  add Transform and GlobalTransform ([ed3a7cb3](https://github.com/zkat/big-brain/commit/ed3a7cb3c03e27b76b374f75ac179f29c979e4cf))

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
