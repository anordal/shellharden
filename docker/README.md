h2. Build the Docker image:

```sh
  docker build -t shellharden:my_tag -f Dockerfile .
```

This creates an image that only includes shellharden.

h2. Build the Docker image:

```sh
  docker build -t shellharden:my_tag -f Dockerfile . --target=pipeline
```

This creates an alpine image that includes /usr/local/cargo/bin/shellharden

h2. Use the Docker image:

```sh
  docker run --rm -v $PWD:/mnt -w /mnt shellharden:my_tag <shellharden arguments>
```
