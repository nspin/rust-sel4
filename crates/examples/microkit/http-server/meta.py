#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

"""
Generates the system description and sDDF configuration structures for the HTTP server example.

There are no sDDF virtualisers in this system. Each driver is connected directly to the server,
using the virtualiser-facing side of the sDDF driver protocols. Drivers refer to client data
regions by I/O address. Because clients refer to the same buffers by offset into those regions,
we give each data region an I/O address of 0 in the driver's device resources.

The sDDF driver class subsystems in acacia_sddf create virtualisers and use C drivers, so we
only use their configuration structure factories, which do not depend on subsystem state.

Also writes a manifest whose lines are of the form "<elf> <section> <blob>", listing the
configuration blobs to patch into protection domain ELF files.
"""

import argparse
import os
import pathlib
from types import SimpleNamespace

from acacia import (
    Channel,
    ConventionalIRQ,
    IRQ,
    MemoryRegion,
    ProtectionDomain,
    System,
)
from acacia.configstruct import ConfigStructResolver

from acacia_sddf import BOARDS, sDDFBlk, sDDFEthernet, sDDFTimer
from acacia_sddf.blk import BLK_PROTOCOL_MAGIC, BLK_QUEUE_ENTRY_SZ
from acacia_sddf.net import NET_BUFFER_SIZE, NET_MAGIC, sDDFMAC
from acacia_sddf.sddf import (
    DeviceRegionResourceFactory,
    DeviceResourcesFactory,
    RegionResourceFactory,
)

DEVICE_MAGIC = 'sDDF' + chr(0x1)

PAGE_SIZE = 0x1000

DEFAULT_STACK_SIZE = 0x10_000

PL031_MMIO_PADDR = 0x9_010_000
PL031_IRQ = 34

SP804_MMIO_PADDR = 0x9_0d0_000
SP804_IRQ = 43

VIRTIO_MMIO_PADDR = 0xa_003_000
VIRTIO_NET_MMIO_OFFSET = 0xe00
VIRTIO_NET_IRQ = 79
VIRTIO_BLK_MMIO_OFFSET = 0xc00
VIRTIO_BLK_IRQ = 78

VIRTIO_DRIVER_DMA_SIZE = 0x200_000

# Must match the QEMU command line
MAC_ADDRESS = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56]

NET_QUEUE_CAPACITY = 512

BLK_QUEUE_CAPACITY = 128
BLK_STORAGE_INFO_REGION_SIZE = 0x1000
BLK_DATA_SIZE = 0x200_000


def page_round_up(n: int) -> int:
    return (n + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--board', required=True, choices=[b.name for b in BOARDS])
    parser.add_argument('--elf-dir', required=True, type=pathlib.Path)
    parser.add_argument('--out-dir', required=True, type=pathlib.Path)
    args = parser.parse_args()
    run(args)


def run(args):
    board = next(filter(lambda b: b.name == args.board, BOARDS))
    sdf = System(board.arch, board.paddr_top)

    server = ProtectionDomain(
        sdf,
        'http_server',
        'microkit-http-server-example-server.elf',
        priority=1,
        stack_size=DEFAULT_STACK_SIZE,
    )
    pl031_driver = ProtectionDomain(
        sdf,
        'pl031_driver',
        'microkit-http-server-example-pl031-driver.elf',
        priority=2,
        stack_size=DEFAULT_STACK_SIZE,
    )
    virtio_blk_driver = ProtectionDomain(
        sdf,
        'virtio_blk_driver',
        'microkit-http-server-example-virtio-blk-driver.elf',
        priority=3,
        stack_size=DEFAULT_STACK_SIZE,
    )
    virtio_net_driver = ProtectionDomain(
        sdf,
        'virtio_net_driver',
        'microkit-http-server-example-virtio-net-driver.elf',
        priority=4,
        stack_size=DEFAULT_STACK_SIZE,
    )
    sp804_driver = ProtectionDomain(
        sdf,
        'sp804_driver',
        'microkit-http-server-example-sp804-driver.elf',
        priority=5,
        stack_size=DEFAULT_STACK_SIZE,
    )

    config_structs = []

    # Device resources are generated after assembly, once physical memory regions have addresses.
    # Each entry is (driver, [(map, offset)], [irq_id], [extra device region]).
    device_resources = []

    def device_irq(pd, irq):
        return pd.add_irq(ConventionalIRQ(irq, IRQ.Trigger.LEVEL))

    def data_region(map):
        return DeviceRegionResourceFactory(RegionResourceFactory(map), 0)

    pl031_mmio = MemoryRegion(sdf, 'pl031_mmio', PAGE_SIZE, paddr=PL031_MMIO_PADDR, cached=False)

    device_resources.append((
        pl031_driver,
        [(pl031_driver.create_automap(pl031_mmio, 'rw'), 0)],
        [device_irq(pl031_driver, PL031_IRQ)],
        [],
    ))

    # The RTC protocol is not part of sDDF
    Channel(
        sdf,
        Channel.End(server, can_notify=False, can_pp=True, setvar_id='rtc_driver_channel_id'),
        Channel.End(pl031_driver, can_notify=False, setvar_id='client_channel_id'),
    )

    sp804_mmio = MemoryRegion(sdf, 'sp804_mmio', PAGE_SIZE, paddr=SP804_MMIO_PADDR, cached=False)

    device_resources.append((
        sp804_driver,
        [(sp804_driver.create_automap(sp804_mmio, 'rw'), 0)],
        [device_irq(sp804_driver, SP804_IRQ)],
        [],
    ))

    timer_channel = Channel(
        sdf,
        Channel.End(server, can_pp=True),
        Channel.End(sp804_driver),
    )

    config_structs.append(sDDFTimer.timer_client_config_factory(
        None,
        server,
        timer_channel.id_for_pd(server),
    ))

    virtio_mmio = MemoryRegion(
        sdf, 'virtio_mmio', PAGE_SIZE, paddr=VIRTIO_MMIO_PADDR, cached=False,
    )

    net_queue_size = page_round_up(8 + 16 * NET_QUEUE_CAPACITY)
    net_data_size = NET_QUEUE_CAPACITY * NET_BUFFER_SIZE

    net_channel = Channel(sdf, Channel.End(server), Channel.End(virtio_net_driver))

    def net_connection(name, data_perms):
        free = MemoryRegion(sdf, f'net_{name}_free', net_queue_size)
        active = MemoryRegion(sdf, f'net_{name}_active', net_queue_size)
        data = MemoryRegion(sdf, f'net_{name}_data', net_data_size)
        resources = {}
        for pd, perms in [(server, data_perms), (virtio_net_driver, 'rw')]:
            resources[pd] = (
                sDDFEthernet.net_connection_resource_factory(
                    None,
                    pd.create_automap(free, 'rw'),
                    pd.create_automap(active, 'rw'),
                    net_channel.id_for_pd(pd),
                    NET_QUEUE_CAPACITY,
                ),
                pd.create_automap(data, perms),
            )
        return resources

    net_rx = net_connection('rx', 'r')
    net_tx = net_connection('tx', 'rw')

    config_structs.append(sDDFEthernet.net_client_config_factory(
        None,
        server,
        NET_MAGIC,
        net_rx[server][0],
        net_rx[server][1],
        net_tx[server][0],
        net_tx[server][1],
        sDDFMAC(MAC_ADDRESS),
    ))

    config_structs.append(sDDFEthernet.net_driver_config_factory(
        SimpleNamespace(driver=virtio_net_driver),
        net_rx[virtio_net_driver][0],
        net_tx[virtio_net_driver][0],
    ))

    virtio_net_driver_dma = MemoryRegion(
        sdf, 'virtio_net_driver_dma', VIRTIO_DRIVER_DMA_SIZE, physical=True,
    )

    # Order must match the driver
    device_resources.append((
        virtio_net_driver,
        [
            (virtio_net_driver.create_automap(virtio_mmio, 'rw'), VIRTIO_NET_MMIO_OFFSET),
            (virtio_net_driver.create_automap(virtio_net_driver_dma, 'rw'), 0),
        ],
        [device_irq(virtio_net_driver, VIRTIO_NET_IRQ)],
        [
            data_region(net_rx[virtio_net_driver][1]),
            data_region(net_tx[virtio_net_driver][1]),
        ],
    ))

    blk_channel = Channel(sdf, Channel.End(server), Channel.End(virtio_blk_driver))

    blk_storage_info = MemoryRegion(sdf, 'blk_storage_info', BLK_STORAGE_INFO_REGION_SIZE)
    blk_queue_size = page_round_up(BLK_QUEUE_CAPACITY * BLK_QUEUE_ENTRY_SZ)
    blk_req_queue = MemoryRegion(sdf, 'blk_req_queue', blk_queue_size)
    blk_resp_queue = MemoryRegion(sdf, 'blk_resp_queue', blk_queue_size)
    blk_data = MemoryRegion(sdf, 'blk_data', BLK_DATA_SIZE)

    blk_connections = {}
    blk_data_maps = {}
    for pd, storage_info_perms in [(server, 'r'), (virtio_blk_driver, 'rw')]:
        blk_connections[pd] = sDDFBlk.blk_connection_resource_factory(
            None,
            pd.create_automap(blk_storage_info, storage_info_perms),
            pd.create_automap(blk_req_queue, 'rw'),
            pd.create_automap(blk_resp_queue, 'rw'),
            blk_channel.id_for_pd(pd),
            BLK_QUEUE_CAPACITY,
        )
        blk_data_maps[pd] = pd.create_automap(blk_data, 'rw')

    config_structs.append(sDDFBlk.blk_client_config_factory(
        None,
        server,
        BLK_PROTOCOL_MAGIC,
        blk_connections[server],
        blk_data_maps[server],
    ))

    config_structs.append(sDDFBlk.blk_driver_config_factory(
        None,
        virtio_blk_driver,
        BLK_PROTOCOL_MAGIC,
        blk_connections[virtio_blk_driver],
    ))

    virtio_blk_driver_dma = MemoryRegion(
        sdf, 'virtio_blk_driver_dma', VIRTIO_DRIVER_DMA_SIZE, physical=True,
    )

    # Order must match the driver
    device_resources.append((
        virtio_blk_driver,
        [
            (virtio_blk_driver.create_automap(virtio_mmio, 'rw'), VIRTIO_BLK_MMIO_OFFSET),
            (virtio_blk_driver.create_automap(virtio_blk_driver_dma, 'rw'), 0),
        ],
        [device_irq(virtio_blk_driver, VIRTIO_BLK_IRQ)],
        [data_region(blk_data_maps[virtio_blk_driver])],
    ))

    args.out_dir = args.out_dir.resolve()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    sdf.write_xml_file(args.out_dir / 'http-server.system')

    for pd, maps_offsets, irqs, extra_regions in device_resources:
        s = DeviceResourcesFactory(DEVICE_MAGIC, maps_offsets, irqs, pd.prog_image)
        s.fields['regions'].extend(extra_regions)
        s.fields['num_regions'] = len(s.fields['regions'])
        config_structs.append(s)

    # Acacia reads the layouts of configuration structures from the DWARF of the ELF files
    os.chdir(args.elf_dir)
    resolver = ConfigStructResolver(args.out_dir.resolve())
    resolver.add_structs(config_structs)
    resolver.resolve_and_create_all()

    with open(args.out_dir / 'patches.txt', 'w') as f:
        for s in config_structs:
            blob = f'{s.target_file.removesuffix(".elf")}_{s.section_name}.data'
            f.write(f'{s.target_file} {s.section_name} {blob}\n')


if __name__ == '__main__':
    main()
