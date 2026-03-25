# Changelog

## [0.4.1](https://github.com/erikmunkby/lazytask/compare/lazytask-v0.4.0...lazytask-v0.4.1) (2026-03-25)


### Bug Fixes

* **tui:** parse version from GitHub release tags with lazytask- prefix ([832d3b1](https://github.com/erikmunkby/lazytask/commit/832d3b1a839f87c866fa6e1d59f86bdd6524879b))

## [0.4.0](https://github.com/erikmunkby/lazytask/compare/lazytask-v0.3.0...lazytask-v0.4.0) (2026-03-25)


### Features

* **tui:** auto-refresh tasks every 3 seconds ([793e649](https://github.com/erikmunkby/lazytask/commit/793e649da111ceaf3ae38a017984de70519887af))
* **tui:** show capacity indicators in task list title ([4abc5ff](https://github.com/erikmunkby/lazytask/commit/4abc5ff5bd339d5f8bd6b6c0ad99a3096d920517))


### Bug Fixes

* **cli:** parse bullet lines before headers in LEARNINGS.md ([e3146fc](https://github.com/erikmunkby/lazytask/commit/e3146fce5a60c8e1163c3a9cf415c89ed9046d31))

## [0.3.0](https://github.com/erikmunkby/lazytask/compare/lazytask-v0.2.0...lazytask-v0.3.0) (2026-03-09)


### Features

* **cli:** decouple learnings from task completion ([ecae8a9](https://github.com/erikmunkby/lazytask/commit/ecae8a9eee4797bef95ed850496b46854742f27a))
* **cli:** make done_reflection prompt configurable via lazytask.toml ([0a9923b](https://github.com/erikmunkby/lazytask/commit/0a9923b8f729287a5dcfac4a8fb8b7897af70ea9))
* **tui:** automatic version check ([077359d](https://github.com/erikmunkby/lazytask/commit/077359d8134d80566587148caffcb8010808295b))


### Bug Fixes

* **cli:** slimmer responses ([05935a1](https://github.com/erikmunkby/lazytask/commit/05935a14a8e712494d22ce1ba283b2029132f42c))
* **cli:** strengthen reflection prompt against safe-learning bias ([d17c555](https://github.com/erikmunkby/lazytask/commit/d17c55509675c9986022f35814a894bb5123d854))

## [0.2.0](https://github.com/erikmunkby/lazytask/compare/lazytask-v0.1.0...lazytask-v0.2.0) (2026-02-27)


### Features

* **cli:** add init --upgrade refresh mode ([4210ab9](https://github.com/erikmunkby/lazytask/commit/4210ab9a58bf1a428179a9f01d053dadeaf9d05d))
* **cli:** add schema-driven config backfill and task retention ttl ([a7e9a25](https://github.com/erikmunkby/lazytask/commit/a7e9a251f2ccef9f9e83c7cc2af601fa48120634))
* **cli:** treat discard as terminal for ai workflows ([574b5ff](https://github.com/erikmunkby/lazytask/commit/574b5ff7081a202809ab89ae8f9bbf12469620da))
* **TUI:** edit tasks ([82cfc99](https://github.com/erikmunkby/lazytask/commit/82cfc99c2271b3709e87795eb9b4520ab6fda2de))
* **tui:** enlarge create modal and wrap long lines ([125ce00](https://github.com/erikmunkby/lazytask/commit/125ce006a82d9328f572055e292296274ba5a698))
* **tui:** group completed tasks below active ([2f576c8](https://github.com/erikmunkby/lazytask/commit/2f576c8016cd6380d96ae6c8074d6b2c55d49059))
* **tui:** hide Created column in task table ([64654e5](https://github.com/erikmunkby/lazytask/commit/64654e55c7f02b544b00ceafe5a4a9d6514f39a3))
* **tui:** open selected tasks in editor ([31fa1ef](https://github.com/erikmunkby/lazytask/commit/31fa1efda6a75893c7fc4f6e3242482aaa0f4991))
* **tui:** simplify footer hints and add keybindings overlay ([38ef78c](https://github.com/erikmunkby/lazytask/commit/38ef78c8f5ac40740a456e7304b45944e37745e4))


### Bug Fixes

* **cli:** make learn workflow action-first ([52a9440](https://github.com/erikmunkby/lazytask/commit/52a9440adc292a42a28473a97ded3d17a065db68))
* **tui:** polish create/edit modal placeholders ([38ea73b](https://github.com/erikmunkby/lazytask/commit/38ea73bb480dacbf7f65a7e9a66a8e222193e6c6))
* **tui:** render cursor at correct spot ([4c1f49a](https://github.com/erikmunkby/lazytask/commit/4c1f49a892e7155966f3ac9fd20b3efde2e0d140))
