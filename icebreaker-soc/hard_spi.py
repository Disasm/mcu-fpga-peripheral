from litex.soc.integration.doc import AutoDoc, ModuleDoc
from litex.soc.interconnect.csr import AutoCSR
from migen import *
from litex.soc.interconnect import wishbone


class HardSPIWrapper(Module):
    def __init__(self, instance):
        assert instance in [0, 1]

        # CSn
        self.mcsnoe = Signal(4)
        self.mcsno = Signal(4)
        self.scsn = Signal()

        # SCK
        self.scko = Signal()
        self.scki = Signal()
        self.sckoe = Signal()

        # MISO
        self.mi = Signal()
        self.so = Signal()
        self.soe = Signal()

        # MOSI
        self.mo = Signal()
        self.si = Signal()
        self.moe = Signal()

        # System Bus
        self.sb_addr = Signal(4)
        self.sb_di = Signal(8)
        self.sb_do = Signal(8)
        self.sb_rw = Signal()
        self.sb_stb = Signal()
        self.sb_ack = Signal()

        self.specials += Instance(
            "SB_SPI",
            i_SBCLKI=ClockSignal(),
            i_SBRWI=self.sb_rw,
            i_SBSTBI=self.sb_stb,
            i_SBADRI0=self.sb_addr[0],
            i_SBADRI1=self.sb_addr[1],
            i_SBADRI2=self.sb_addr[2],
            i_SBADRI3=self.sb_addr[3],
            i_SBADRI4=0,
            i_SBADRI5=instance,
            i_SBADRI6=0,
            i_SBADRI7=0,
            i_SBDATI0=self.sb_di[0],
            i_SBDATI1=self.sb_di[1],
            i_SBDATI2=self.sb_di[2],
            i_SBDATI3=self.sb_di[3],
            i_SBDATI4=self.sb_di[4],
            i_SBDATI5=self.sb_di[5],
            i_SBDATI6=self.sb_di[6],
            i_SBDATI7=self.sb_di[7],
            o_SBDATO0=self.sb_do[0],
            o_SBDATO1=self.sb_do[1],
            o_SBDATO2=self.sb_do[2],
            o_SBDATO3=self.sb_do[3],
            o_SBDATO4=self.sb_do[4],
            o_SBDATO5=self.sb_do[5],
            o_SBDATO6=self.sb_do[6],
            o_SBDATO7=self.sb_do[7],
            o_SBACKO=self.sb_ack,
            i_MI=self.mi,
            o_SO=self.so,
            o_SOE=self.soe,
            i_SI=self.si,
            o_MO=self.mo,
            o_MOE=self.moe,
            i_SCKI=self.scki,
            o_SCKO=self.scko,
            o_SCKOE=self.sckoe,
            i_SCSNI=self.scsn,
            o_MCSNO0=self.mcsno[0],
            o_MCSNO1=self.mcsno[1],
            o_MCSNO2=self.mcsno[2],
            o_MCSNO3=self.mcsno[3],
            o_MCSNOE0=self.mcsnoe[0],
            o_MCSNOE1=self.mcsnoe[1],
            o_MCSNOE2=self.mcsnoe[2],
            o_MCSNOE3=self.mcsnoe[3],
            p_BUS_ADDR74=["0b0000", "0b0010"][instance],
        )


class HardSPISlavePeripheral(Module):
    def __init__(self, spi_pads, instance=0):
        assert instance in [0, 1]
        self.bus = bus = wishbone.Interface()

        self.submodules.spi = spi = HardSPIWrapper(instance)

        self.specials += Instance(
            "SB_IO",
            io_PACKAGE_PIN=spi_pads.cs_n,
            i_OUTPUT_ENABLE=0,
            o_D_IN_0=spi.scsn,
            p_PIN_TYPE=0b101001,
            p_PULLUP=1,
        )

        self.specials += Instance(
            "SB_IO",
            io_PACKAGE_PIN=spi_pads.miso,
            i_OUTPUT_ENABLE=spi.soe,
            i_D_OUT_0=spi.so,
            p_PIN_TYPE=0b101001,
        )

        self.comb += [
            spi.scki.eq(spi_pads.clk),
            spi.si.eq(spi_pads.mosi),
        ]

        self.comb += [
            spi.sb_addr.eq(bus.adr[0:4]),
            spi.sb_di.eq(bus.dat_w[0:8]),
            spi.sb_rw.eq(bus.we),
            spi.sb_stb.eq(bus.cyc),
            If(bus.cyc, bus.dat_r.eq(spi.sb_do)),
            bus.ack.eq(spi.sb_ack),
        ]


class HardSPISlave(Module):
    def __init__(self, spi_pads, instance=0):
        self.submodules.spi = spi = HardSPIWrapper(instance)
        self.start = Signal()
        self.done = Signal()
        self.byte = Signal()
        self.mosi = Signal(8)
        self.miso = Signal(8)
        self.dbg_status = Signal()

        self.specials += Instance(
            "SB_IO",
            io_PACKAGE_PIN=spi_pads.cs_n,
            i_OUTPUT_ENABLE=0,
            o_D_IN_0=spi.scsn,
            p_PIN_TYPE=0b101001,
            p_PULLUP=1,
        )

        self.specials += Instance(
            "SB_IO",
            io_PACKAGE_PIN=spi_pads.miso,
            i_OUTPUT_ENABLE=spi.soe,
            i_D_OUT_0=spi.so,
            p_PIN_TYPE=0b101001,
        )

        self.comb += [
            spi.scki.eq(spi_pads.clk),
            spi.si.eq(spi_pads.mosi),
        ]

        sr_busy = Signal()

        self.submodules.fsm = fsm = FSM(reset_state="RESET")
        fsm.act("RESET",
            spi.sb_stb.eq(1),
            spi.sb_rw.eq(1),
            spi.sb_addr.eq(0b1001),  # SPICR1
            spi.sb_di.eq(0b10000000),  # Enable SPI
            If(spi.sb_ack, NextState("IDLE"))
        )
        fsm.act("IDLE",
            NextValue(self.byte, 0),
            NextState("READ_STATUS")
        )
        fsm.act("READ_STATUS",
            spi.sb_stb.eq(1),
            spi.sb_rw.eq(0),
            spi.sb_addr.eq(0b1100),  # SPISR
            self.dbg_status.eq(1),
            If(spi.sb_ack,
                NextValue(sr_busy, spi.sb_do[6]),
                If(spi.sb_do[4],
                    NextState("UPDATE_TX_START")
                ).Elif(spi.sb_do[3],
                    NextState("UPDATE_RX_START")
                ).Else(
                    NextState("IDLE")
                )
            )
        )
        fsm.act("UPDATE_TX_START", NextState("UPDATE_TX"))
        fsm.act("UPDATE_TX",
            spi.sb_stb.eq(1),
            spi.sb_rw.eq(1),
            spi.sb_addr.eq(0b1101),  # SPITXDR
            spi.sb_di.eq(self.miso),
            If(spi.sb_ack, NextState("IDLE"))
        )
        fsm.act("UPDATE_RX_START", NextState("UPDATE_RX"))
        fsm.act("UPDATE_RX",
            spi.sb_stb.eq(1),
            spi.sb_rw.eq(0),
            spi.sb_addr.eq(0b1110),  # SPIRXDR
            If(spi.sb_ack,
                NextValue(self.mosi, spi.sb_do),
                NextValue(self.byte, 1),
                NextState("IDLE")
            )
        )

        sr_busy_d = Signal()
        self.sync += sr_busy_d.eq(sr_busy)
        self.comb += [
            self.start.eq(sr_busy & ~sr_busy_d),
            self.done.eq(~sr_busy),
        ]
