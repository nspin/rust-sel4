import argparse
import json
import shutil
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--tree', type=Path)
    parser.add_argument('--sel4-source', type=Path)
    parser.add_argument('--scratch', type=Path)
    args = parser.parse_args()
    build_kernels(args.tree, args.sel4_source, args.scratch)


def build_kernels(tree, sel4_source, scratch):
    by_hash_dir = tree / 'configs' / 'by-hash'
    for config in by_hash_dir.iterdir():
        with (config / 'config' / 'misc.json').open() as f:
            misc = json.load(f)

        cross_compiler_prefix = misc['cross_compiler_prefix']
        install_prefix = config / 'seL4'

        subprocess.run(
            [
                'cmake',
                '-DCMAKE_TOOLCHAIN_FILE=gcc.cmake',
                f'-DCROSS_COMPILER_PREFIX={cross_compiler_prefix}',
                f'-DCMAKE_INSTALL_PREFIX={install_prefix}',
                '-C', config / 'config' / 'seL4.settings.cmake',
                '-G', 'Ninja',
                '-S', sel4_source,
                '-B', scratch,
            ],
            check=True,
        )

        subprocess.run(
            ['ninja', '-C', scratch, 'all'],
            check=True,
        )

        subprocess.run(
            ['ninja', '-C', scratch, 'install'],
            check=True,
        )

        if misc['requires_i386_kernel']:
            subprocess.run(
                [
                    f'{cross_compiler_prefix}objcopy',
                    '-O', 'elf32-i386',
                    install_prefix / 'bin' / 'kernel.elf',
                    config / 'kernel32.elf',
                ],
                check=True,
            )

        shutil.rmtree(scratch)


if __name__ == '__main__':
    main()
