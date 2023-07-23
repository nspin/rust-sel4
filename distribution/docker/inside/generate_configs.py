import argparse
import hashlib
import itertools
import json
import struct
from dataclasses import dataclass
from pathlib import Path

SHORT_HASH_LENGTH = 12


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-o', '--out-dir', type=Path)
    args = parser.parse_args()
    generate_configs(args.out_dir)


def generate_configs(out_dir):
    out_dir.mkdir(parents=True)
    (out_dir / 'by-hash').mkdir()
    (out_dir / 'by-alias').mkdir()
    for config in CONFIGS:
        config.realize(out_dir)


@dataclass
class Config(dict):
    entries: dict
    aliases: list[str]

    def hash(self):
        return hash_strings(itertools.chain.from_iterable(self.entries.items()))

    def short_hash(self):
        return self.hash()[:SHORT_HASH_LENGTH]

    def realize(self, out_dir: Path):
        config_dir = out_dir / 'by-hash' / self.short_hash() / 'config'
        config_dir.mkdir(parents=True)
        for k, v in self.entries.items():
            (config_dir / k).write_text(v)
        for alias in self.aliases:
            link_source = out_dir / 'by-alias' / alias
            link_source.symlink_to(Path('..') / 'by-hash' / self.short_hash())


def hash_strings(it):
    m = hashlib.sha256()
    for s in it:
        m.update(struct.pack('!i', len(s)))
        m.update(s.encode('utf-8'))
    return m.hexdigest()


class ConfigGenerator:

    @classmethod
    def generate(cls, *args, **kwargs):
        return cls(*args, **kwargs).do_generate()

    def do_generate(self):
        entries = {}
        entries['seL4.settings.cmake'] = self.collect_kernel_config()
        entries['kernel-loader.config.json'] = self.collect_kernel_loader_config()
        entries['misc.json'] = json.dumps({
            'cross_compiler_prefix': self.cross_compiler_prefix,
            'sel4_minimal_target': self.sel4_minimal_target,
            'bare_metal_target': self.bare_metal_target,
            'requires_kernel_loader': self.requires_kernel_loader,
            'requires_i386_kernel': self.requires_i386_kernel,
        })
        qemu_command = self.qemu_command()
        if qemu_command is not None:
            cmd = ' \\\n    '.join(itertools.chain(qemu_command, ['"$@"']))
            entries['simulate.sh'] = f'exec {cmd}\n'
        return entries

    def collect_kernel_config(self):
        return ''.join(self.kernel_config())

    def collect_kernel_loader_config(self):
        return json.dumps(dict(self.kernel_loader_config()), sort_keys=True, indent=4)

    def kernel_config(self):
        yield from ()

    def kernel_loader_config(self):
        yield from ()

    def qemu_command(self):
        return None


def mk_cmake_set(k, v):
    if isinstance(v, str):
        v_cmake = v
        ty = 'STRING'
    elif isinstance(v, bool):
        v_cmake = 'TRUE' if v else 'FALSE'
        ty = 'BOOL'
    else:
        raise Exception(v)
    return f'set({k} {v_cmake} CACHE {ty} "")\n'


class Base(ConfigGenerator):
    def __init__(self):
        super().__init__()
        self.requires_kernel_loader = True
        self.requires_i386_kernel = False

    def kernel_config(self):
        yield mk_cmake_set('KernelVerificationBuild', False)
        yield mk_cmake_set('KernelRootCNodeSizeBits', '14')


class AArch64(Base):
    def __init__(self):
        super().__init__()
        self.cross_compiler_prefix = 'aarch64-linux-gnu-'
        self.sel4_minimal_target = 'aarch64-sel4-minimal'
        self.bare_metal_target = 'aarch64-unknown-none'


class X86_64(Base):
    def __init__(self):
        super().__init__()
        self.cross_compiler_prefix = ''
        self.sel4_minimal_target = 'x86_64-sel4-minimal'
        self.bare_metal_target = 'x86_64-unknown-none'
        self.requires_kernel_loader = False
        self.requires_i386_kernel = True


class QEMUArmVirt(AArch64):
    def __init__(
        self,
        num_cores: int = 1,
        mcs: bool = False,
        cpu: str = 'cortex-a57',
        hypervisor: bool = False,
        **kwargs,
    ):
        super().__init__(**kwargs)
        self.num_cores = num_cores
        self.mcs = mcs
        self.cpu = cpu
        self.hypervisor = hypervisor

    def kernel_config(self):
        yield from super().kernel_config()
        yield mk_cmake_set('ARM_CPU', self.cpu)
        yield mk_cmake_set('KernelArch', 'arm')
        yield mk_cmake_set('KernelSel4Arch', 'aarch64')
        yield mk_cmake_set('KernelPlatform', 'qemu-arm-virt')
        yield mk_cmake_set('KernelMaxNumNodes', str(self.num_cores))
        yield mk_cmake_set('KernelIsMCS', self.mcs)
        yield mk_cmake_set('KernelArmHypervisorSupport', self.hypervisor)

    def qemu_command(self):
        return [
            'qemu-system-aarch64',
            '-machine', f'virt,virtualization=on',
            '-cpu', self.cpu,
            '-smp', str(self.num_cores),
            '-m', '1024',
            '-nographic',
            '-serial', 'mon:stdio',
        ]


class PC99(X86_64):
    def __init__(
        self,
        num_cores: int = 1,
        mcs: bool = False,
        **kwargs,
    ):
        super().__init__(**kwargs)
        self.num_cores = num_cores
        self.mcs = mcs

    def kernel_config(self):
        yield from super().kernel_config()
        yield mk_cmake_set('KernelArch', 'x86')
        yield mk_cmake_set('KernelSel4Arch', 'x86_64')
        yield mk_cmake_set('KernelPlatform', 'pc99')
        yield mk_cmake_set('KernelMaxNumNodes', str(self.num_cores))
        yield mk_cmake_set('KernelIsMCS', self.mcs)
        yield mk_cmake_set('KernelFSGSBase', 'msr')
        yield mk_cmake_set('KernelSupportPCID', False)
        yield mk_cmake_set('KernelIOMMU', False)
        yield mk_cmake_set('KernelFPU', 'FXSAVE')

    def qemu_command(self):
        opts = Opts()
        opts.disable('vme')
        opts.enable('pdpe1gb')
        opts.disable('xsave')
        opts.disable('xsaveopt')
        opts.disable('xsavec')
        opts.disable('fsgsbase')
        opts.disable('invpcid')
        opts.enable('syscall')
        opts.enable('lm')
        opts = opts.finalize()
        return [
            'qemu-system-x86_64',
            '-cpu', f'Nehalem,{opts}enforce',
            '-m', 'size=512M',
            '-nographic',
            '-serial', 'mon:stdio',
        ]


class Opts:
    def __init__(self):
        self.acc = []

    def enable(self, opt):
        self.acc.append(f'+{opt}')

    def disable(self, opt):
        self.acc.append(f'-{opt}')

    def finalize(self):
        return ''.join((f'{opt},' for opt in self.acc))


CONFIGS = [
    Config(QEMUArmVirt.generate(hypervisor=True), ['qemu-arm-virt']),
    Config(QEMUArmVirt.generate(hypervisor=True, mcs=True), ['qemu-arm-virt-with-mcs']),
    Config(PC99.generate(), ['pc99']),
    Config(PC99.generate(mcs=True), ['pc99-with-mcs']),
]

if __name__ == '__main__':
    main()
