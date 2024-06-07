#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import argparse
import json
import sys
from jinja2 import Template

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--template', type=argparse.FileType('r'), required=True)
    parser.add_argument('--board', required=True)
    parser.add_argument('-o', type=argparse.FileType('w'), required=True)
    args = parser.parse_args()
    run(args)

def run(args):
    template = Template(args.template.read())
    context = mk_context(args.board)
    rendered = template.render(context)
    args.o.write(rendered)

def mk_context(board):
    context = {}

    if board == 'qemu_virt_aarch64':
        context['uart_mmio_phys_addr'] = 0x9000000
        context['uart_irq'] = 33
    elif board == 'zcu102':
        context['uart_mmio_phys_addr'] = 0xff000000
        context['uart_irq'] = 53
    else:
        raise Exception('unsupported configuration')

    return context

if __name__ == '__main__':
    main()
