#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

"""
Generates the system description and sDDF configuration structures for the banscii example.

The serial driver is connected directly to the assistant, without sDDF serial virtualisers.

Also writes a manifest whose lines are of the form "<elf> <section> <blob>", listing the
configuration blobs to patch into protection domain ELF files.
"""

import argparse
import os
import pathlib
from typing import List
from acacia import System, Arch, aarch64, ProtectionDomain, MemoryRegion, Channel, ConventionalIRQ
from acacia.configstruct import ConfigStructResolver
from dataclasses import dataclass

from acacia_sddf import sDDFSerial
from acacia_sddf.serial import SERIAL_PROTOCOL_MAGIC
from acacia_sddf.sddf import DeviceResourcesFactory


@dataclass
class Board:
    name: str
    arch: Arch
    paddr_top: int
    serial_mmio_phys_addr: int
    serial_irq: int


BOARDS: List[Board] = [
    Board(
        name='qemu_virt_aarch64',
        arch=aarch64,
        paddr_top=0x6_0000_000,
        serial_mmio_phys_addr=0x9_000_000,
        serial_irq=33,
    ),
]


DEVICE_MAGIC = 'sDDF' + chr(0x1)

DEFAULT_STACK_SIZE = 0x10_000

SERIAL_QUEUE_SIZE = 0x1_000
# Must be a power of two
SERIAL_DATA_SIZE = 0x1_000
SERIAL_BAUD_RATE = 115200


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

    serial_mmio = MemoryRegion(sdf, 'serial_mmio', 0x1_000, paddr=board.serial_mmio_phys_addr)
    assistant_to_artist = MemoryRegion(sdf, 'assistant_to_artist', 0x4_000)
    artist_to_assistant = MemoryRegion(sdf, 'artist_to_assistant', 0x4_000)

    serial_driver = ProtectionDomain(
        sdf,
        'serial_driver',
        'banscii-serial-driver.elf',
        priority=3,
        stack_size=DEFAULT_STACK_SIZE,
    )
    serial_mmio_map = serial_driver.create_automap(serial_mmio, 'rw')
    serial_irq = serial_driver.add_irq(
        ConventionalIRQ(board.serial_irq, ConventionalIRQ.Trigger.LEVEL)
    )

    assistant = ProtectionDomain(
        sdf,
        'assistant',
        'banscii-assistant.elf',
        priority=1,
        stack_size=DEFAULT_STACK_SIZE,
    )
    assistant.create_automap(
        artist_to_assistant,
        'r',
        setvar_vaddr='region_in_start',
        setvar_size='region_in_size',
    )
    assistant.create_automap(
        assistant_to_artist,
        'rw',
        setvar_vaddr='region_out_start',
        setvar_size='region_out_size',
    )

    artist = ProtectionDomain(
        sdf,
        'artist',
        'banscii-artist.elf',
        priority=2,
        stack_size=DEFAULT_STACK_SIZE,
    )
    artist.create_automap(
        assistant_to_artist,
        'r',
        setvar_vaddr='region_in_start',
        setvar_size='region_in_size',
    )
    artist.create_automap(
        artist_to_assistant,
        'rw',
        setvar_vaddr='region_out_start',
        setvar_size='region_out_size',
    )

    def serial_connection(name, producer):
        queue = MemoryRegion(sdf, f'serial_{name}_queue', SERIAL_QUEUE_SIZE)
        data = MemoryRegion(sdf, f'serial_{name}_data', SERIAL_DATA_SIZE)
        channel = Channel(sdf, Channel.End(serial_driver), Channel.End(assistant))
        return {
            pd: sDDFSerial.serial_connection_resource_factory(
                None,
                pd.create_automap(queue, 'rw'),
                pd.create_automap(data, 'rw' if pd is producer else 'r'),
                channel.id_for_pd(pd),
            )
            for pd in [serial_driver, assistant]
        }

    serial_rx = serial_connection('rx', serial_driver)
    serial_tx = serial_connection('tx', assistant)

    config_structs = [
        sDDFSerial.serial_driver_config_factory(
            None,
            serial_driver,
            SERIAL_PROTOCOL_MAGIC,
            SERIAL_BAUD_RATE,
            1,
            serial_tx[serial_driver],
            serial_rx[serial_driver],
        ),
        sDDFSerial.serial_client_config_factory(
            None,
            assistant,
            SERIAL_PROTOCOL_MAGIC,
            serial_tx[assistant],
            serial_rx[assistant],
        ),
    ]

    Channel(
        sdf,
        Channel.End(assistant, can_pp=True, setvar_id='artist_channel_id'),
        Channel.End(artist, setvar_id='assistant_channel_id'),
    )

    args.out_dir = args.out_dir.resolve()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    sdf.write_xml_file(args.out_dir / 'banscii.system')

    config_structs.append(DeviceResourcesFactory(
        DEVICE_MAGIC,
        [(serial_mmio_map, 0)],
        [serial_irq],
        serial_driver.prog_image,
    ))

    # Acacia reads the layouts of configuration structures from the DWARF of the ELF files
    os.chdir(args.elf_dir)
    resolver = ConfigStructResolver(args.out_dir)
    resolver.add_structs(config_structs)
    resolver.resolve_and_create_all()

    with open(args.out_dir / 'patches.txt', 'w') as f:
        for s in config_structs:
            blob = f'{s.target_file.removesuffix(".elf")}_{s.section_name}.data'
            f.write(f'{s.target_file} {s.section_name} {blob}\n')


if __name__ == '__main__':
    main()
