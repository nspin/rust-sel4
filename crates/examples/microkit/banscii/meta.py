#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import argparse
from typing import List
from acacia import System, Arch, aarch64, ProtectionDomain, MemoryRegion, Channel, ConventionalIRQ
from dataclasses import dataclass


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


DEFAULT_STACK_SIZE = 0x10_000


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--board', required=True, choices=[b.name for b in BOARDS])
    parser.add_argument('-o', required=True)
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
    serial_driver.create_automap(
        serial_mmio,
        'rw',
        setvar_vaddr='serial_register_block',
    )
    serial_driver.add_irq(
        ConventionalIRQ(
            board.serial_irq,
            ConventionalIRQ.Trigger.LEVEL,
            setvar_id='serial_irq_id',
        )
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

    Channel(
        sdf,
        Channel.End(serial_driver, setvar_id='assistant_channel_id'),
        Channel.End(assistant, can_pp=True, setvar_id='serial_driver_channel_id'),
    )

    Channel(
        sdf,
        Channel.End(assistant, can_pp=True, setvar_id='artist_channel_id'),
        Channel.End(artist, setvar_id='assistant_channel_id'),
    )

    sdf.write_xml_file(args.o)


if __name__ == '__main__':
    main()
