#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

# Expects SDDF, MICROKIT_SDK, MICROKIT_BOARD, and MICROKIT_CONFIG.

.DEFAULT_GOAL := all

SUPPORTED_BOARDS := $(MICROKIT_BOARD)
TOOLCHAIN := clang
SDDF_CUSTOM_LIBC := 1

SDDF_OS := microkit

include $(SDDF)/tools/make/sddf.mk
include $(SDDF)/libco/libco.mk
include $(SDDF)/sound/components/sound_components.mk

CFLAGS += \
	-I$(SDDF)/include \
	-I$(SDDF)/include/$(SDDF_OS) \
	-Wno-unused-function

# For snippets which only provide rules for linking (e.g. benchmark.mk)
vpath %.c $(SDDF)

LDFLAGS := -L$(BOARD_DIR)/lib
LIBS := --start-group -lmicrokit -Tmicrokit.ld libsddf_util_debug.a --end-group

LIBRARIES := \
	libsddf_util.a \
	libsddf_util_debug.a

# libco's x86_64 implementation includes assert.h, which sDDF's libc does not provide
ifneq ($(ARCH),x86_64)
LIBRARIES += libco.a
endif

# gpu_virt.elf is omitted because it is configured by a system-specific header (gpu_config.h).
# network_arp.elf is omitted because it was not ported to runtime configuration, and still includes
# ethernet_config.h, which no longer exists.
COMPONENTS := \
	$(SERIAL_IMAGES) \
	$(filter-out network_arp.elf,$(NETWORK_IMAGES)) \
	$(BLK_IMAGES) \
	$(SND_IMAGES) \
	i2c_virt.elf \
	benchmark.elf \
	idle.elf

# The board snippet includes a snippet for each of the board's drivers. Each such snippet, other
# than those for ethernet drivers, is named after the driver it builds.
#
# gpu_driver.elf is omitted because the virtio GPU driver does not build (it includes the
# nonexistent sddf/virtio/virtio.h).
DRIVER_SNIPPETS := $(filter %_driver.mk %_driver_pnk.mk,$(notdir $(MAKEFILE_LIST)))
DRIVERS := $(sort \
	$(ETH_DRIV) \
	$(ETH_DRIV_1) \
	$(patsubst %.mk,%.elf,$(patsubst %_pnk.mk,%.mk,$(filter-out eth_driver.mk gpu_driver.mk,$(DRIVER_SNIPPETS)))))

# Not all snippets make libsddf_util_debug.a, which is in LIBS, a prerequisite.
$(COMPONENTS) $(DRIVERS): libsddf_util_debug.a

.PHONY: all
all: $(LIBRARIES) $(COMPONENTS) $(DRIVERS)

.PHONY: install
install: all
	mkdir -p $(PREFIX)/lib $(PREFIX)/elf $(PREFIX)/include
	cp $(LIBRARIES) $(PREFIX)/lib
	cp $(COMPONENTS) $(DRIVERS) $(PREFIX)/elf
	cp -R $(SDDF)/include/sddf $(SDDF)/include/$(SDDF_OS)/os $(PREFIX)/include
