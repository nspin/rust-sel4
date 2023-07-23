ifeq ($(TREE),)
	$(error "TREE must be set")
endif

TARGET_DIR ?= target

tree := $(TREE)
by_config_hash := $(tree)/configs/by-hash
bin := $(tree)/bin

target_dir := $(TARGET_DIR)

workspace_dir := ../../..

config_hashes := $(shell ls $(by_config_hash))

lookup_misc_config = $(shell jq .$(2) $(by_config_hash)/$(1)/config/misc.json)

# "true" or "false"
requires_kernel_loader = $(shell jq .requires_kernel_loader $(by_config_hash)/$(1)/config/misc.json)

common_cargo_env_cross = \
	CC=$(call lookup_misc_config,$*,cross_compiler_prefix)gcc \
	SEL4_PREFIX=$(abspath $(by_config_hash))/$*/seL4 \
	SEL4_KERNEL_LOADER_CONFIG=$(abspath $(by_config_hash))/$*/config/kernel-loader.config.json

common_cargo_args = \
	--release \
	--target-dir $(abspath $(target_dir))

# TODO not working yet, output is still dynamically linked
common_cargo_env_native = \
	RUSTFLAGS="-C target-feature=+crt-static"

common_cargo_args_native = \
	$(common_cargo_args) \
	--target $(shell uname -m)-unknown-linux-musl \
	--out-dir $(abspath $(bin))

common_cargo_args_cross = \
	$(common_cargo_args) \
	-Z build-std=core,alloc,compiler_builtins \
	-Z build-std-features=compiler-builtins-mem \
	--out-dir $(abspath $(dir $@))

.PHONY: default
default: all

kernel_loader_crate := sel4-kernel-loader
kernel_loader_bin := $(kernel_loader_crate)

$(by_config_hash)/%/$(kernel_loader_bin): $(by_config_hash)/%/$(kernel_loader_bin).intermediate
	true

.INTERMEDIATE: $(by_config_hash)/%/$(kernel_loader_bin).intermediate
$(by_config_hash)/%/$(kernel_loader_bin).intermediate:
	cd $(workspace_dir) && \
		$(common_cargo_env_cross) \
		cargo build \
			$(common_cargo_args_cross) \
			--target $(call lookup_misc_config,$*,bare_metal_target) \
			-p $(kernel_loader_crate)

kernel_loader_cli_crate := sel4-kernel-loader-add-payload
kernel_loader_cli := $(bin)/$(kernel_loader_cli_crate)
kernel_loader_cli_intermediate := $(kernel_loader_cli).intermediate

$(kernel_loader_cli): $(kernel_loader_cli_intermediate)

.INTERMDIATE: $(kernel_loader_cli_intermediate)
$(kernel_loader_cli_intermediate):
	cd $(workspace_dir) && \
		$(common_cargo_env_native) \
		cargo build \
			$(common_cargo_args_native) \
			-p $(kernel_loader_cli_crate)

capdl_initializer_crate := sel4-capdl-initializer
capdl_initializer_bin := $(capdl_initializer_crate).elf

$(by_config_hash)/%/$(capdl_initializer_bin): $(by_config_hash)/%/$(capdl_initializer_bin).intermediate
	true

.INTERMEDIATE: $(by_config_hash)/%/$(capdl_initializer_bin).intermediate
$(by_config_hash)/%/$(capdl_initializer_bin).intermediate:
	cd $(workspace_dir) && \
		$(common_cargo_env_cross) \
		cargo build \
			$(common_cargo_args_cross) \
			--target $(call lookup_misc_config,$*,sel4_minimal_target) \
			-p $(capdl_initializer_crate)

capdl_initializer_cli_crate := sel4-capdl-initializer-add-spec
capdl_initializer_cli := $(bin)/$(capdl_initializer_cli_crate)
capdl_initializer_cli_intermediate := $(capdl_initializer_cli).intermediate

$(capdl_initializer_cli): $(capdl_initializer_cli_intermediate)

.INTERMDIATE: $(capdl_initializer_cli_intermediate)
$(capdl_initializer_cli_intermediate):
	cd $(workspace_dir) && \
		$(common_cargo_env_native) \
		cargo build \
			$(common_cargo_args_native) \
			-p $(capdl_initializer_cli_crate)

all_bins := \
	$(kernel_loader_cli) \
	$(capdl_initializer_cli) \
	$(foreach config_hash,$(config_hashes), \
		$(by_config_hash)/$(config_hash)/$(capdl_initializer_bin) \
		$(if $(filter true,$(call requires_kernel_loader,$(config_hash))), \
			$(by_config_hash)/$(config_hash)/$(kernel_loader_bin)))

.PHONY: all
all: $(all_bins)

.PHONY: clean
clean:
	rm -rf $(target_dir) $(all_bins)
