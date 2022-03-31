docker build . -t rust-local-builder
docker run --rm -it --volume $(PWD):/rust/src --entrypoint bash rust-local-builder