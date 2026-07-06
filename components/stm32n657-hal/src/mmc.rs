use core::marker::PhantomData;

use bitflags::Flags;

use crate::{
    rcc::Rcc,
    sdmmc::{
        self, Disabled, DpsmState, Enabled, Error, FIFO_SIZE, PowerCtrl, Resp, ResponseBits, SdMmc,
        SdMmcMaster, TransferDir, TransferMode,
    },
};

#[derive(Default)]
pub struct CardInfo {
    card_type: CardType,
    /// Rel Card Add
    rca: u16,
    class: u32,
    block_number: u32,
    block_size: u32,
    log_block_number: u32,
    log_block_size: u32,
}

pub struct MmcMaster<P, S> {
    sdmmc: SdMmcMaster<P, S>,
    state: State,
    card_info: CardInfo,
    cid: [u32; 4],
    csd: [u32; 4],
    ext_csd: [u32; 128],
    errorstate: Error,
    _state: PhantomData<S>,
}

#[derive(Default)]
enum CardType {
    #[default]
    LowCapacity,
    HighCapacity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Reset = 0,
    Ready = 1,
    Timeout = 2,
    Busy = 3,
    Programming = 4,
    Receiving = 5,
    Transfer = 6,
    Error = 0xF,
}

impl<P: SdMmc> MmcMaster<P, Disabled> {
    pub fn new(peripheral: P) -> Self {
        Self {
            errorstate: Error::empty(),
            sdmmc: SdMmcMaster::new(peripheral),
            state: State::Reset,
            card_info: CardInfo::default(),
            cid: [0; 4],
            csd: [0; 4],
            ext_csd: [0; 128],
            _state: PhantomData,
        }
    }

    pub fn enable(self, _rcc: &Rcc) -> Result<MmcMaster<P, Enabled>, Error> {
        let init = sdmmc::SdMMCInit {
            clock_edge: sdmmc::ClockEdge::Rising,
            clock_power_save: sdmmc::ClockPowerSave::Disable,
            bus_wide: sdmmc::BusWidth::OneBit,
            hardware_flow_control: sdmmc::HardwareFlowControl::Disable,
            clock_div: 0, // TODO get proper clock divider
            is_transceiver_present: 0,
        };
        let sdmmc = self.sdmmc.enable(init);
        let mut this = MmcMaster {
            errorstate: Error::empty(),
            sdmmc,
            state: State::Ready,
            card_info: CardInfo::default(),
            cid: [0; 4],
            csd: [0; 4],
            ext_csd: [0; 128],
            _state: PhantomData,
        };
        this.sdmmc.power_on();

        this.power_on()?;

        this.init_card()?;

        todo!()
    }
}

#[derive(Default)]
struct Csd {
    ///*!< CSD structure                         
    csd_struct: u8,
    ///System specification version          
    sys_spec_version: u8,
    /// Reserved                              
    reserved1: u8,
    /// Data read access time 1               
    taac: u8,
    /// Data read access time 2 in CLK cycles
    nsac: u8,
    /// Max. bus clock frequency              
    max_bus_clk_frec: u8,
    /// Card command classes                  
    card_comd_classes: u16,
    ///Max. read data block length           
    rd_block_len: u8,
    /// Partial blocks for read allowed       
    part_block_read: u8,
    /// Write block misalignment              
    wr_block_misalign: u8,
    /// Read block misalignment               
    rd_block_misalign: u8,
    /// DSR implemented                       
    dsr_impl: u8,
    /// Reserved                              
    reserved2: u8,
    /// Device Size                           
    device_size: u32,
    ///Max. read current @ VDD min           
    max_rd_current_vdd_min: u8,
    /// Max. read current @ VDD max           
    max_rd_current_vdd_max: u8,
    /// Max. write current @ VDD min          
    max_wr_current_vdd_min: u8,
    /// Max. write current @ VDD max          
    max_wr_current_vdd_max: u8,
    /// Device size multiplier                
    device_size_mul: u8,
    /// Erase group size                      
    erase_gr_size: u8,
    /// Erase group size multiplier           
    erase_gr_mul: u8,
    /// Write protect group size              
    wr_protect_gr_size: u8,
    /// Write protect group enable            
    wr_protect_gr_enable: u8,
    /// Manufacturer default ECC              
    man_defl_ec_c: u8,
    /// Write speed factor                    
    wr_speed_fact: u8,
    /// Max. write data block length          
    max_wr_block_len: u8,
    /// Partial blocks for write allowed      
    write_block_pa_partial: u8,
    /// Reserved                              
    reserved3: u8,
    /// Content protection application        
    content_protect_appli: u8,
    /// File format group                     
    file_format_group: u8,
    /// Copy flag (OTP)                       
    copy_flag: u8,
    /// Permanent write protection            
    perm_wr_protect: u8,
    /// Temporary write protection            
    temp_wr_protect: u8,
    /// File format                           
    file_format: u8,
    /// ECC code                              
    ecc: u8,
    /// CSD CRC                               
    csd_crc: u8,
    /// Always 1                              
    reserved4: u8,
}
impl<P: SdMmc> MmcMaster<P, Enabled> {
    fn power_on(&mut self) -> Result<(), Error> {
        self.sdmmc.cmd_go_idle_state()?;

        let mut valid_voltage = 0;
        let mut count = 0;
        let mut response = ResponseBits::empty();
        while valid_voltage == 0 {
            count += 1;
            if count == 0xFFFF {
                return Err(Error::INVALID_VOLTRANGE);
            }
            self.sdmmc.cmd_op_condition(0xC0000080)?;
            response = self.sdmmc.get_response(Resp::Resp1);
            valid_voltage = response.bits() >> 31;
        }

        if response.bits() & 0xFF000000 == 0xC0000000 {
            self.card_info.card_type = CardType::HighCapacity;
        } else {
            self.card_info.card_type = CardType::LowCapacity;
        }
        Ok(())
    }

    fn init_card(&mut self) -> Result<(), Error> {
        if self.sdmmc.power() == PowerCtrl::Off {
            return Err(Error::REQUEST_NOT_APPLICABLE);
        }

        self.sdmmc.cmd_send_cid()?;
        self.cid[0] = self.sdmmc.get_response(Resp::Resp1).bits();
        self.cid[1] = self.sdmmc.get_response(Resp::Resp2).bits();
        self.cid[2] = self.sdmmc.get_response(Resp::Resp3).bits();
        self.cid[3] = self.sdmmc.get_response(Resp::Resp4).bits();
        self.card_info.rca = self.sdmmc.cmd_set_rel_add()?;

        self.sdmmc.cmd_send_csd()?;
        self.csd[0] = self.sdmmc.get_response(Resp::Resp1).bits();
        self.csd[1] = self.sdmmc.get_response(Resp::Resp2).bits();
        self.csd[2] = self.sdmmc.get_response(Resp::Resp3).bits();
        self.csd[3] = self.sdmmc.get_response(Resp::Resp4).bits();

        self.card_info.class = self.sdmmc.get_response(Resp::Resp2).bits() >> 20;

        self.sdmmc
            .cmd_select_deselect((self.card_info.rca as u32) << 16)?;

        let _csd = self.get_card_csd()?;

        if let Err(err) = self
            .sdmmc
            .cmd_send_status((self.card_info.rca as u32) << 16)
        {
            self.errorstate |= err;
        }

        self.ext_csd = self.get_card_ext_csd()?;

        if let Err(err) = self
            .sdmmc
            .cmd_send_status((self.card_info.rca as u32) << 16)
        {
            self.errorstate |= err;
        }

        Ok(())
    }

    fn get_card_csd(&mut self) -> Result<Csd, Error> {
        let mut csd = Csd {
            csd_struct: ((self.csd[0] & 0xC0000000) >> 30)
                .try_into()
                .expect("value to fit in u8"),
            sys_spec_version: ((self.csd[0] & 0x3C000000) >> 26)
                .try_into()
                .expect("value to fit in u8"),
            reserved1: ((self.csd[0] & 0x03000000) >> 24)
                .try_into()
                .expect("value to fit in u8"),
            taac: ((self.csd[0] & 0x00FF0000) >> 16)
                .try_into()
                .expect("value to fit in u8"),
            nsac: ((self.csd[0] & 0x0000FF00) >> 8)
                .try_into()
                .expect("value to fit in u8"),
            max_bus_clk_frec: (self.csd[0] & 0x000000FF)
                .try_into()
                .expect("value to fit in u8"),
            card_comd_classes: ((self.csd[1] & 0xFFF00000) >> 20)
                .try_into()
                .expect("value to fit in u8"),
            rd_block_len: ((self.csd[1] & 0x000F0000) >> 16)
                .try_into()
                .expect("value to fit in u8"),
            part_block_read: ((self.csd[1] & 0x00008000) >> 15)
                .try_into()
                .expect("value to fit in u8"),
            wr_block_misalign: ((self.csd[1] & 0x00004000) >> 14)
                .try_into()
                .expect("value to fit in u8"),
            rd_block_misalign: ((self.csd[1] & 0x00002000) >> 13)
                .try_into()
                .expect("value to fit in u8"),
            dsr_impl: ((self.csd[1] & 0x00001000) >> 12)
                .try_into()
                .expect("value to fit in u8"),
            erase_gr_size: ((self.csd[2] & 0x00004000) >> 14)
                .try_into()
                .expect("value to fit in u8"),
            erase_gr_mul: ((self.csd[2] & 0x00003F80) >> 7)
                .try_into()
                .expect("value to fit in u8"),
            wr_protect_gr_size: (self.csd[2] & 0x0000007F)
                .try_into()
                .expect("value to fit in u8"),
            wr_protect_gr_enable: ((self.csd[3] & 0x80000000) >> 31)
                .try_into()
                .expect("value to fit in u8"),
            man_defl_ec_c: ((self.csd[3] & 0x60000000) >> 29)
                .try_into()
                .expect("value to fit in u8"),
            wr_speed_fact: ((self.csd[3] & 0x1C000000) >> 26)
                .try_into()
                .expect("value to fit in u8"),
            max_wr_block_len: ((self.csd[3] & 0x03C00000) >> 22)
                .try_into()
                .expect("value to fit in u8"),
            write_block_pa_partial: ((self.csd[3] & 0x00200000) >> 21)
                .try_into()
                .expect("value to fit in u8"),
            content_protect_appli: ((self.csd[3] & 0x00010000) >> 16)
                .try_into()
                .expect("value to fit in u8"),
            file_format_group: ((self.csd[3] & 0x00008000) >> 15)
                .try_into()
                .expect("value to fit in u8"),
            copy_flag: ((self.csd[3] & 0x00004000) >> 14)
                .try_into()
                .expect("value to fit in u8"),
            perm_wr_protect: ((self.csd[3] & 0x00002000) >> 13)
                .try_into()
                .expect("value to fit in u8"),
            temp_wr_protect: ((self.csd[3] & 0x00001000) >> 12)
                .try_into()
                .expect("value to fit in u8"),
            file_format: ((self.csd[3] & 0x00000C00) >> 10)
                .try_into()
                .expect("value to fit in u8"),
            ecc: ((self.csd[3] & 0x00000300) >> 8)
                .try_into()
                .expect("value to fit in u8"),
            csd_crc: ((self.csd[3] & 0x000000FE) >> 1)
                .try_into()
                .expect("value to fit in u8"),
            reserved3: 0,
            reserved4: 1,
            ..Default::default()
        };

        let block_number = self.read_ext_csd(212, 0x0FFFFFFF)?;

        match self.card_info.card_type {
            CardType::LowCapacity => {
                csd.device_size =
                    ((self.csd[1] & 0x000003FF) << 2) | ((self.csd[2] & 0xC0000000) >> 30);
                csd.max_rd_current_vdd_min = ((self.csd[2] & 0x38000000) >> 27) as u8;
                csd.max_rd_current_vdd_max = ((self.csd[2] & 0x07000000) >> 24) as u8;
                csd.max_wr_current_vdd_min = ((self.csd[2] & 0x00E00000) >> 21) as u8;
                csd.max_wr_current_vdd_max = ((self.csd[2] & 0x001C0000) >> 18) as u8;
                csd.device_size_mul = ((self.csd[2] & 0x00038000) >> 15) as u8;
                self.card_info.block_number =
                    (csd.device_size + 1) * (1 << ((csd.device_size_mul & 0x7) + 2));
                self.card_info.block_size = 1 << csd.rd_block_len & 0xF;

                self.card_info.log_block_number =
                    self.card_info.block_number * self.card_info.block_size / BLOCK_SIZE;
                self.card_info.log_block_size = BLOCK_SIZE;
            }
            CardType::HighCapacity => {
                self.card_info.block_number = block_number;
                self.card_info.log_block_number = block_number;
                self.card_info.block_size = BLOCK_SIZE;
                self.card_info.log_block_size = BLOCK_SIZE;
            }
        }
        Ok(csd)
    }

    fn read_ext_csd(&mut self, field_index: u16, _timeout: u32) -> Result<u32, Error> {
        let mut ret = 0;
        self.errorstate.clear();
        self.sdmmc
            .peripheral
            .dctrl()
            .write(|w| unsafe { w.bits(0) });

        self.sdmmc.config_data(sdmmc::DataInit {
            data_time_out: 0xFFFFFFFF,
            data_len: 512,
            data_block_size: sdmmc::DataBlockSize::B512,
            transfer_dir: TransferDir::ToSdMmc,
            transfer_mode: TransferMode::Block,
            dpsm: DpsmState::Enable,
        });
        self.sdmmc.cmd_trans_enable();
        if let Err(err) = self.sdmmc.cmd_send_ext_csd(0) {
            self.errorstate |= err;
            self.sdmmc.clear_static_flags();
            return Err(err);
        }
        let mut star;

        let mut dataremaining = 512;

        let mut i = 0;
        while {
            star = self.sdmmc.peripheral.star().read();
            !(star.rxoverr().bit()
                | star.dcrcfail().bit()
                | star.dtimeout().bit()
                | star.dataend().bit())
        } {
            if star.rxfifohf().bit() && dataremaining >= FIFO_SIZE {
                for count in 0..FIFO_SIZE / 4 {
                    let tmp = self.sdmmc.read_fifo();
                    if i + count == field_index as usize / 4 {
                        ret = tmp;
                    }
                }
                i += 8;
                dataremaining -= FIFO_SIZE;
            }

            if false
            /* TODO: timeout */
            {
                self.sdmmc.clear_static_flags();
                self.errorstate |= Error::TIMEOUT;
                self.state = State::Ready;
                return Err(Error::TIMEOUT);
            }
        }

        self.sdmmc.cmd_trans_disable();

        let star = self.sdmmc.peripheral.star().read();
        if star.dtimeout().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::TIMEOUT;
            self.state = State::Ready;
            return Err(Error::TIMEOUT);
        } else if star.dcrcfail().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::DATA_CRC_FAIL;
            self.state = State::Ready;
            return Err(Error::DATA_CRC_FAIL);
        } else if star.rxoverr().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::RX_OVERRUN;
            self.state = State::Ready;
            return Err(Error::RX_OVERRUN);
        }

        if let Err(err) = self
            .sdmmc
            .cmd_send_status((self.card_info.rca as u32) << 16)
        {
            self.errorstate |= err;
        }

        self.sdmmc.clear_static_flags();
        self.state = State::Ready;
        Ok(ret)
    }

    fn get_card_ext_csd(&mut self) -> Result<[u32; 128], Error> {
        assert_eq!(self.state, State::Ready);
        self.errorstate.clear();
        self.state = State::Busy;
        self.sdmmc
            .peripheral
            .dctrl()
            .write(|w| unsafe { w.bits(0) });

        let config = sdmmc::DataInit {
            data_time_out: 0xFFFFFFFF,
            data_len: 512,
            data_block_size: sdmmc::DataBlockSize::B512,
            transfer_dir: TransferDir::ToSdMmc,
            transfer_mode: TransferMode::Block,
            dpsm: DpsmState::Disable,
        };
        self.sdmmc.config_data(config);
        self.sdmmc.cmd_trans_enable();
        let mut tmpbuf = [0; 128];

        if let Err(err) = self.sdmmc.cmd_send_ext_csd(0) {
            self.sdmmc.clear_static_flags();
            self.errorstate |= err;
            self.state = State::Ready;
            return Err(err);
        }

        let mut dataremaining = 512;
        let mut offset = 0;

        let mut star;
        while {
            star = self.sdmmc.peripheral.star().read();
            !(star.rxoverr().bit()
                | star.dcrcfail().bit()
                | star.dtimeout().bit()
                | star.dataend().bit())
        } {
            if star.rxfifohf().bit() && dataremaining >= FIFO_SIZE {
                for _ in 0..FIFO_SIZE / 4 {
                    tmpbuf[offset] = self.sdmmc.read_fifo();
                    offset += 1;
                }
                dataremaining -= FIFO_SIZE;
            }
            // TODO: timeout
        }

        self.sdmmc.cmd_trans_disable();

        let star = self.sdmmc.peripheral.star().read();
        if star.dtimeout().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::TIMEOUT;
            self.state = State::Ready;
            return Err(Error::TIMEOUT);
        } else if star.dcrcfail().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::DATA_CRC_FAIL;
            self.state = State::Ready;
            return Err(Error::DATA_CRC_FAIL);
        } else if star.rxoverr().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::RX_OVERRUN;
            self.state = State::Ready;
            return Err(Error::RX_OVERRUN);
        }

        self.sdmmc.clear_static_flags();
        self.state = State::Ready;

        Ok(tmpbuf)
    }
}

const BLOCK_SIZE: u32 = 512;
