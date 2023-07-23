work_root := ../..

id := rust-sel4

image_namespace := nspin
image_repository := $(id)
image_tag := latest
image_tag_complete := $(image_namespace)/$(image_repository):$(image_tag)

container_name := $(id)

used_image_tag := ghcr.io/$(image_tag_complete)
# used_image_tag := $(image_repository)

.PHONY: none
none:

.PHONY: build
build:
	DOCKER_BUILDKIT=1 \
		docker build \
			-t $(used_image_tag) \
			-f Dockerfile \
			../..

.PHONY: run
run: build
	docker run --rm \
		--name $(container_name) \
		--mount type=bind,src=$(abspath $(work_root)),dst=/work \
		-it $(used_image_tag) \
		bash

.PHONY: push
push: build
	docker push $(used_image_tag)
