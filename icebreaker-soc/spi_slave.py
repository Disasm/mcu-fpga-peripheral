# Copyright (c) 2019-2020 Florent Kermarrec <florent@enjoy-digital.fr>
# Copyright (c) 2020 Vadim Kaushan <admin@disasm.info>
# SPDX-License-Identifier: BSD-2-Clause
from litex.soc.integration.doc import AutoDoc, ModuleDoc
from litex.soc.interconnect.csr import AutoCSR
from litex.soc.interconnect import wishbone
from migen import *
from migen.genlib.cdc import MultiReg
from hard_spi import HardSPISlave


# SPI Slave ----------------------------------------------------------------------------------------
# This module is a modified version of `litex.soc.cores.spi.SPISlave`

class SPISlave(Module):
    """4-wire SPI Slave

    Provides a simple and minimal hardware SPI Slave with CPOL=0, CPHA=0 and build time configurable
    data_width.
    """
    pads_layout = [("clk", 1), ("cs_n", 1), ("mosi", 1), ("miso", 1)]

    def __init__(self, pads):
        if pads is None:
            pads = Record(self.pads_layout)
        if not hasattr(pads, "cs_n"):
            pads.cs_n = Signal()
        self.pads       = pads

        self.start    = Signal()
        self.byte     = Signal()
        self.length   = Signal(3)
        self.done     = Signal()
        self.irq      = Signal()
        self.mosi     = Signal(8)
        self.miso     = Signal(8)
        self.cs       = Signal()

        # # #

        clk  = Signal()
        cs   = Signal()
        mosi = Signal()
        miso = Signal()

        # IOs <--> Internal (input resynchronization) ----------------------------------------------
        self.specials += [
            MultiReg(pads.clk, clk),
            MultiReg(~pads.cs_n, cs),
            MultiReg(pads.mosi, mosi),
        ]
        self.comb += pads.miso.eq(miso)

        # Clock detection --------------------------------------------------------------------------
        clk_d = Signal()
        clk_rise = Signal()
        clk_fall = Signal()
        self.sync += clk_d.eq(clk)
        self.comb += clk_rise.eq(clk & ~clk_d)
        self.comb += clk_fall.eq(~clk & clk_d)

        # Control FSM ------------------------------------------------------------------------------
        self.submodules.fsm = fsm = FSM(reset_state="IDLE")
        fsm.act("IDLE",
            If(cs,
                self.start.eq(1),
                NextValue(self.length, 0),
                NextState("XFER")
            ).Else(
                self.done.eq(1)
            )
        )
        fsm.act("XFER",
            If(~cs,
                self.irq.eq(1),
                NextState("IDLE")
            ),
            NextValue(self.length, self.length + clk_fall)
        )
        byte = Signal()
        self.comb += byte.eq(clk_fall & (self.length == 7))
        self.sync += self.byte.eq(byte)

        # Master In Slave Out (MISO) generation (generated on spi_clk falling edge) ----------------
        miso_data = Signal(8)
        self.sync += \
            If(self.start | self.byte,
                miso_data.eq(self.miso)
            ).Elif(cs & clk_fall,
                miso_data.eq(Cat(Signal(), miso_data[:-1]))
            )
        self.comb += miso.eq(miso_data[-1])

        # Master Out Slave In (MOSI) capture (captured on spi_clk rising edge) ---------------------
        mosi_data = Signal(8)
        self.sync += [
            If(cs & clk_rise,
                mosi_data.eq(Cat(mosi, mosi_data[:-1]))
            ),
            If(byte, self.mosi.eq(mosi_data))
        ]


class SPIBridge(Module, AutoCSR, AutoDoc):
    def __init__(self, spi_pads):
        # Documentation
        self.intro = ModuleDoc("SPI slave driver")

        self.wishbone = bus = wishbone.Interface()
        self.submodules.spi = spi = HardSPISlave(spi_pads)

        spi_counter = Signal(4)
        spi_dword_mosi = Signal(32)
        spi_dword_miso = Signal(32)
        self.sync += \
            If(spi.start,
                spi_counter.eq(0),
            ).Elif(spi.byte,
                spi_counter.eq(spi_counter + 1),
                spi_dword_mosi.eq(Cat(spi_dword_mosi[8:], spi.mosi)),
                spi_dword_miso.eq(Cat(spi_dword_miso[8:], Signal(8))),
            )

        address = Signal(16)
        address_hi = Signal(16)

        command = Signal(8)

        self.comb += spi.miso.eq(spi_dword_miso[:8])

        self.submodules.fsm = fsm = FSM(reset_state="IDLE")
        fsm.act("IDLE",
            If(spi.start, NextState("COMMAND"))
        )
        fsm.act("COMMAND",
            If(spi.done,
                NextState("IDLE")
            ).Else(
                If(spi_counter == 1,
                    NextValue(command, spi_dword_mosi[-8:]),
                ),
                If(spi_counter == 3,
                    NextValue(address, spi_dword_mosi[-16:]),
                    If(command == 0x03, NextState("READ"))
                ),
                If((spi_counter == 7) and command == 0x02,
                    #NextValue(data, spi_dword_mosi),
                    NextState("WRITE"),
                )
            ),
        )
        fsm.act("READ",
            bus.cyc.eq(1),
            bus.stb.eq(1),
            bus.we.eq(0),
            bus.adr.eq(Cat(address, address_hi)),
            bus.sel.eq(2 ** len(bus.sel) - 1),
            If(bus.ack,
                NextValue(spi_dword_miso, bus.dat_r),
                NextState("IDLE")
            )
        )
        fsm.act("WRITE",
            bus.cyc.eq(1),
            bus.stb.eq(1),
            bus.we.eq(1),
            bus.adr.eq(Cat(address, address_hi)),
            bus.dat_w.eq(spi_dword_mosi),
            bus.sel.eq(2 ** len(bus.sel) - 1),
            If(bus.ack,
                NextState("IDLE")
            )
        )
