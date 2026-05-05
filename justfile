cargo +ARGS:
	bazel run @rules_rust//tools/upstream_wrapper:cargo -- {{ARGS}}

cargo-clippy *ARGS:
	bazel run @rules_rust//tools/upstream_wrapper:cargo_clippy -- {{ARGS}}

rustup +ARGS:
	bazel run @rules_rust//tools/upstream_wrapper:rustup -- {{ARGS}}

rustc +ARGS:
	bazel run @rules_rust//tools/upstream_wrapper:rustc -- {{ARGS}}

gazelle:
	bazel run //:gazelle

test +ARGS:
	bazel test {{ARGS}}

alias g := gazelle
