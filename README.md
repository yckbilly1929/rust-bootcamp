# rust-bootcamp

My monorepo for [rust learning materials](https://github.com/tyr-rust-bootcamp).

## Development guide

1. git hook on client-side

    ```bash
      chmod -R +x .githooks
      git config core.hooksPath .githooks
    ```

1. git subtree to include assignment template

    ```bash
      git subtree add -P app/simple-redis --squash https://github.com/tyr-rust-bootcamp/02-simple-redis master
    ```
