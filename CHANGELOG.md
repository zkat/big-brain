# `big-brain` Release Changelog

<a name="0.11.0"></a>
## 0.11.0 (2022-05-05)

### Features

* **deps:** upgrade to Bevy 0.7. (#37) ([38688789](https://github.com/zkat/big-brain/commit/38688789d08547e1cbc0d2a373fc58af39dea360))

<a name="0.10.0"></a>
## 0.10.0 (2022-01-15)

This is my first pass at a significant API improvement. I iterated on it for a
while and this is what I settled on. I look forward to continuing to evolve
this API as I get more feedback and experience with it! Please let me know
what you think!

### Breaking Changes

* **thinker:** stop cancelling actions when they go under Picker thresholds ([4c72b3d1](https://github.com/zkat/big-brain/commit/4c72b3d11eaa42af4b99ccf9ea729306e589ada8))
* **stages:** Strongly typed stages for BigBrainPlugin ([65ca646e](https://github.com/zkat/big-brain/commit/65ca646e3b92b178025591878e2df6a08714880f))
* **builders:** Blanket impls for ActionBuilder and ScorerBuilder when Clone ([8bed75b5](https://github.com/zkat/big-brain/commit/8bed75b54a43c72b53fbf9e2605b942cb2c53214))
* **api:** rename attach and hide it from docs ([6c64df4f](https://github.com/zkat/big-brain/commit/6c64df4fc1211abe19919a3628476b930b6e9919))
* **choice:** return &Choice instead of cloning ([a76dcbb6](https://github.com/zkat/big-brain/commit/a76dcbb67d4f6ae402f03d22e8d526408d8d875f))

### Features

* **example:** update thirst example ([edecc4c9](https://github.com/zkat/big-brain/commit/edecc4c95f76bcd69c042372140486f744f4ccea))

### Bug Fixes

* **hierarchy:** make sure all the hierarchy stuff is set up properly ([372c13e2](https://github.com/zkat/big-brain/commit/372c13e207523ec919c6490682f628f7d21cebea))
* **tests:** update tests ([94e1b1f6](https://github.com/zkat/big-brain/commit/94e1b1f685e6ab0be9d90bae5dfbd648ba87f1de))

<a name="0.9.0"></a>
## 0.9.0 (2022-01-09)

### Features

* **deps:** update for bevy 0.6 (#31) ([b97f9273](https://github.com/zkat/big-brain/commit/b97f9273c5f5eceb010d8fa2b23abb534fb2cee1))
* **perf:** Use SparseSet for ActionState ([5fc08176](https://github.com/zkat/big-brain/commit/5fc081765c1ed8788a7a5d1e940efbc66dc8aa8f))

<a name="0.8.0"></a>
## 0.8.0 (2021-09-19)

### Bug Fixes

* **systems:** Fix steps, add a test and explicit systems ordering (#27) ([f33315c9](https://github.com/zkat/big-brain/commit/f33315c9b7b769a94baab17e3a9df9f5ebe924d2)) (credit: [@payload](https://github.com/payload))

<a name="0.7.0"></a>
## 0.7.0 (2021-09-16)

### Bug Fixes

* **deps:** Don't include Bevy default features when used as a dependency. (#25) ([61558137](https://github.com/zkat/big-brain/commit/615581370a165645795966ac7c878ff492630ba2))

### Features

* **license:** change license to Apache-2.0 ([d7d17772](https://github.com/zkat/big-brain/commit/d7d177729476af8ec1463d8957a35092a336098a))
    * **BREAKING CHANGE**: This is a significant licensing change. Please review.

<a name="0.6.0"></a>
## 0.6.0 (2021-05-20)

#### Features

* **pickers:**  Make choices mod public (#23) ([92034cd0](https://github.com/zkat/big-brain/commit/92034cd04e629723893cfcd7730ce597083da9e7))
* **scorers:**  Added EvaluatingScorer (#24) ([1a1d5b3d](https://github.com/zkat/big-brain/commit/1a1d5b3d17d96a51084418128f0bfebe0ad8c702))

#### Bug Fixes

* **actions:**  Concurrently was not setting its state to Failure ([d4a689f6](https://github.com/zkat/big-brain/commit/d4a689f6c60f509a71fb3a9ae4ca49dad263acab))


<a name="0.5.0"></a>
## 0.5.0 (2021-04-27)

Got a few goodies in this release, mainly focused around composite actions and
scorers, which were apparently broken.

Shout out again to [@piedoomy](https://github.com/piedoomy) for some of these
contributions!

#### Features

* **actions:**  Add new Concurrently composite action ([6c736374](https://github.com/zkat/big-brain/commit/6c736374b4afd60af592a357ad2403304d3638d1))
* **evaluators:**  added inversed linear evaluator helper (#19) ([f871d19e](https://github.com/zkat/big-brain/commit/f871d19e93b6764088d6db5db1947fcb37143868))
* **scorers:**  Added WinningScorer composite scorer (#20) ([748b30ae](https://github.com/zkat/big-brain/commit/748b30aedcb0711f4180a8e24b457f01f0b84f6a))

#### Breaking Changes

* **scorers:** Composite Scorers now all use `.push()` instead of a mixture of `.push()` and `.when()`. Please update any usages of composite scorers ([63bad1fd](https://github.com/zkat/big-brain/commit/63bad1fd2c82eadc88107003dd819f3cfa7530a2)

#### Bug Fixes

* **scorers:**
  *  Scorer builders now properly return themselves ([63bad1fd](https://github.com/zkat/big-brain/commit/63bad1fd2c82eadc88107003dd819f3cfa7530a2)
  *  Fixed error where wrong component for `SumOfScorers` was attached (#21) ([71fd05a6](https://github.com/zkat/big-brain/commit/71fd05a64912b2cc88c76439543ea00a00267303))



<a name="0.4.0"></a>
## 0.4.0 (2021-04-26)

#### Breaking Changes

* **score:**  scores are now 0.0..=1.0, not 0.0..=100.0 ([71f5122e](https://github.com/zkat/big-brain/commit/71f5122e9f5aa5b5965ad67f53ae9850f487d167), breaks [#](https://github.com/zkat/big-brain/issues/))

#### Features

* **evaluators:**  Make all evaluators Clone ([4d5a5121](https://github.com/zkat/big-brain/commit/4d5a512171bf6f850893424c5baad03b0e686c26))


<a name="0.3.5"></a>
## 0.3.5 (2021-04-24)

Previously, if a `Picker` re-picked the same action, and that action had been
set to `Success` or `Failure`, it would just keep running the action in that
state until it was time to switch to a different one.

With this version, that behavior is changed, and `Failure` and `Success`
actions that are re-picked will be respawned entirely (not even reused).

Cheers to [@piedoomy](https://github.com/piedoomy) on Discord for pointing out
this weird behavior!

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
