changelog tag: (ensure-bin-crate "git-cliff")
    git-cliff --prepend CHANGELOG.md {{tag}}

release tag: (ensure-bin-crate "cargo-release")
    cargo release --workspace {{tag}}

ensure-bin-crate name:
    cargo install --locked {{name}}

readme: (ensure-bin-crate "cargo-readme")
    cargo readme -o README.md
    cargo readme -t .github/README.tpl -o .github/README.md