#!/usr/bin/env python3

# This file is Copyright (c) 2019 Sean Cross <sean@xobs.io>
# This file is Copyright (c) 2018 David Shah <dave@ds0.me>
# This file is Copyright (c) 2020 Piotr Esden-Tempski <piotr@esden.net>
# This file is Copyright (c) 2020 Vadim Kaushan <admin@disasm.info>
# License: BSD

# This target was originally based on the Fomu target.

# This variable defines all the external programs that this module
# relies on.  lxbuildenv reads this variable in order to ensure
# the build will finish without exiting due to missing third-party
# programs.
LX_DEPENDENCIES = ["riscv", "icestorm", "yosys", "nextpnr-ice40"]

# Import lxbuildenv to integrate the deps/ directory
import lxbuildenv

import argparse

from litex.build.generic_platform import Subsignal, Pins, IOStandard
from litex.build.lattice.programmer import IceStormProgrammer
from litex.soc.cores.up5kspram import Up5kSPRAM
from litex.soc.integration.builder import Builder, builder_args, builder_argdict
from litex.soc.integration.soc_core import soc_core_args, soc_core_argdict, SoCCore
from litex_boards.platforms.icebreaker import Platform, break_off_pmod
from migen import *
from migen.genlib.resetsync import AsyncResetSynchronizer
from litex.soc.cores.clock import iCE40PLL
from litex.soc.integration.doc import AutoDoc

from leds import Leds
from spi_slave import SPIBridge


class _CRG(Module, AutoDoc):
    """Icebreaker Clock Resource Generator

    The system is clocked by the external 12MHz clock. But if a sys_clk_freq is set to a value
    that is different from the default 12MHz we will feed it through the PLL block and try to
    generate a clock as close as possible to the selected frequency.
    """
    def __init__(self, platform, sys_clk_freq):
        self.clock_domains.cd_sys = ClockDomain()
        self.clock_domains.cd_por = ClockDomain()

        # # #

        # Clocks
        clk12 = platform.request("clk12")
        if sys_clk_freq == 12e6:
            self.comb += self.cd_sys.clk.eq(clk12)
        else:
            self.submodules.pll = pll = iCE40PLL(primitive="SB_PLL40_PAD")
            pll.register_clkin(clk12, 12e6)
            pll.create_clkout(self.cd_sys, sys_clk_freq, with_reset=False)
        platform.add_period_constraint(self.cd_sys.clk, 1e9 / sys_clk_freq)

        # Power On Reset
        self.reset = Signal()
        por_cycles = 8
        por_counter = Signal(log2_int(por_cycles), reset=por_cycles - 1)
        self.comb += self.cd_por.clk.eq(self.cd_sys.clk)
        platform.add_period_constraint(self.cd_por.clk, 1e9 / sys_clk_freq)
        self.sync.por += If(por_counter != 0, por_counter.eq(por_counter - 1))
        self.comb += self.cd_sys.rst.eq(por_counter != 0)
        self.specials += AsyncResetSynchronizer(self.cd_por, self.reset)


# BaseSoC ------------------------------------------------------------------------------------------

class BaseSoC(SoCCore):
    """A SoC on iCEBreaker, without a softcore CPU"""

    # Statically-define the memory map, to prevent it from shifting across various litex versions.
    SoCCore.mem_map = {
        "csr":              0x00000000,
        "sram":             0x00020000,
    }

    def __init__(self, sys_clk_freq, **kwargs):
        platform = Platform()

        kwargs["cpu_type"] = None
        kwargs["with_uart"] = False
        kwargs["with_timer"] = False
        #kwargs["with_ctrl"] = False

        # Force the SRAM size to 0, because we add our own SRAM with SPRAM
        kwargs["integrated_sram_size"] = 0
        kwargs["integrated_rom_size"] = 0

        kwargs["csr_data_width"] = 32

        SoCCore.__init__(self, platform, sys_clk_freq, **kwargs)

        self.submodules.crg = _CRG(platform, sys_clk_freq)

        reset_btn = platform.request("user_btn_n")
        self.comb += self.crg.reset.eq(~reset_btn)

        led = platform.request("user_led_n")
        led2 = platform.request("user_led_n")

        spi_ext = [
            ("spi_slave", 0,
                Subsignal("cs_n", Pins("PMOD1A:0"), IOStandard("LVCMOS33")),
                Subsignal("clk", Pins("PMOD1A:1"), IOStandard("LVCMOS33")),
                Subsignal("miso", Pins("PMOD1A:2"), IOStandard("LVCMOS33")),
                Subsignal("mosi", Pins("PMOD1A:3"), IOStandard("LVCMOS33")),
            ),
        ]
        platform.add_extension(spi_ext)
        spi_pads = platform.request("spi_slave")

        self.submodules.bridge = bridge = SPIBridge(spi_pads)
        self.bus.add_master(name="bridge", master=self.bridge.wishbone)

        # UP5K has single port RAM, which is a dedicated 128 kilobyte block.
        # Use this as CPU RAM.
        # spram_size = 128 * 1024
        # self.submodules.spram = Up5kSPRAM(size=spram_size)
        # self.register_mem("sram", self.mem_map["sram"], self.spram.bus, spram_size)

        # The litex SPI module supports memory-mapped reads, as well as a bit-banged mode
        # for doing writes.
        # spiflash_size = 16 * 1024 * 1024
        # self.submodules.spiflash = SpiFlash(platform.request("spiflash4x"), dummy=6, endianness="little")
        # self.register_mem("spiflash", self.mem_map["spiflash"], self.spiflash.bus, size=spiflash_size)
        # self.add_csr("spiflash")

        # Add ROM linker region
        #self.add_memory_region("rom", self.mem_map["spiflash"] + flash_offset, spiflash_size - flash_offset, type="cached+linker")

        platform.add_extension(break_off_pmod)
        self.submodules.leds = Leds(Cat(
            #platform.request("user_ledr_n"),
            #platform.request("user_ledg_n"),
            platform.request("user_ledr"),
            platform.request("user_ledg", 0),
            platform.request("user_ledg", 1),
            platform.request("user_ledg", 2),
            platform.request("user_ledg", 3)),
            led_polarity=0x00,
            led_name=[
                #["ledr", "The Red LED on the main iCEBreaker board."],
                #["ledg", "The Green LED on the main iCEBreaker board."],
                ["hledr1", "The center Red LED #1 on the iCEBreaker head."],
                ["hledg2", "Green LED #2 on the iCEBreaker head."],
                ["hledg3", "Green LED #3 on the iCEBreaker head."],
                ["hledg4", "Green LED #4 on the iCEBreaker head."],
                ["hledg5", "Green LED #5 on the iCEBreaker head."]])

        self.add_csr("leds")

        assert hasattr(self.platform.toolchain, "build_template")
        if self.platform.toolchain.build_template[0].startswith("yosys "):
            self.platform.toolchain.build_template[0] =\
                self.platform.toolchain.build_template[0].replace("yosys ", "yosys -q ")


def main():
    parser = argparse.ArgumentParser(description="LiteX SoC on iCEBreaker")
    parser.add_argument("--sys-clk-freq", type=float, default=48e6, help="Select system clock frequency")
    parser.add_argument("--document-only", action="store_true", help="Do not build a soc. Only generate documentation.")
    parser.add_argument("--flash", action="store_true", help="Load bitstream")
    builder_args(parser)
    soc_core_args(parser)
    args = parser.parse_args()

    # Create the SOC
    soc = BaseSoC(sys_clk_freq=int(args.sys_clk_freq), **soc_core_argdict(args))

    # Configure command line parameter defaults
    # Don't build software -- we don't include it since we just jump to SPI flash.
    builder_kwargs = builder_argdict(args)
    builder_kwargs["compile_software"] = False

    if args.document_only:
        builder_kwargs["compile_gateware"] = False
    if builder_kwargs["csr_svd"] is None:
        builder_kwargs["csr_svd"] = "../litex-pac/soc.svd"

    # Create and run the builder
    builder = Builder(soc, **builder_kwargs)
    builder.build()

    # If requested load the resulting bitstream onto the iCEBreaker
    if args.flash:
        IceStormProgrammer().flash(0x00000000, "build/icebreaker/gateware/icebreaker.bin")


if __name__ == "__main__":
    main()
