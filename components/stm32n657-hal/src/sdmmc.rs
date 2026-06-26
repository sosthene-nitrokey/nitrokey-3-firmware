use core::{marker::PhantomData, ops::Deref};

use bitflags::bitflags;
use stm32n6::stm32n657::{SDMMC1, SDMMC2, sdmmc1};

use crate::utils::enum_u;

pub trait SdMmc: Deref<Target = sdmmc1::RegisterBlock> {}

impl SdMmc for SDMMC1 {}
impl SdMmc for SDMMC2 {}

/// Specifies the SDMMC_CCK clock transition on which Data and Command change.g
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ClockEdge {
    Raising = 0x0,
    #[doc(alias = "SDMMC_CLKCR_NEGEDGE")]
    Falling = 0x00010000,
}

/// Specifies whether SDMMC Clock output is enabled or disabled when the bus is idle
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ClockPowerSave {
    Disable = 0,
    #[doc(alias = "SDMMC_CLKCR_PWRSAV")]
    Enable = 0x00001000,
}

/// Specifies the SDMMC bus width
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum BusWidth {
    OneBit = 0,
    #[doc(alias = "SDMMC_CLKCR_WIDBUS_0")]
    FourBit = 0x00004000,
    #[doc(alias = "SDMMC_CLKCR_WIDBUS_1")]
    EightBit = 0x00008000,
}

impl BusWidth {
    pub fn is_wide(self) -> bool {
        !matches!(self, Self::OneBit)
    }
}

/// Specifies whether the SDMMC hardware flow control is enabled or disabled
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum HardwareFlowControl {
    Disable = 0,
    #[doc(alias = "SDMMC_CLKCR_HWFC_EN")]
    Enable = 0x00020000,
}

/// Specifies whether there is a transceiver present
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum TransceiverPresent {
    Unknown = 0,
    NotPresent = 1,
    Present = 2,
}

pub struct SdMMCInit {
    pub clock_edge: ClockEdge,
    pub clock_power_save: ClockPowerSave,
    pub bus_wide: BusWidth,
    pub hardware_flow_control: HardwareFlowControl,
    /// Specifies the clock frequency of the SDMMC controller.
    /// This parameter has to be in 0..=1023
    pub clock_div: u16,
    pub is_transceiver_present: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Response {
    No = 0,
    #[doc(alias = "SDMMC_CMD_WAITRESP_0")]
    Short = 0x00000100,
    #[doc(alias = "SDMMC_CMD_WAITRESP")]
    Long = 0x00000300,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum WaitForInterruptState {
    No = 0,
    #[doc(alias = "SDMMC_CMD_WAITINT")]
    It = 0x00000400,
    #[doc(alias = "SDMMC_CMD_WAITPEND")]
    Pend = 0x00000800,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum CpsmState {
    Disable = 0,
    #[doc(alias = "SDMMC_CMD_CPSMEN")]
    Enable = 0x00001000,
}

pub struct SdMmcCmdInit {
    pub argument: u32,
    /// Must be within 0..=64
    pub cmd_index: Command,
    pub response: Response,
    pub wait_for_interrupt: WaitForInterruptState,
    pub cpsm: CpsmState,
}

const SDMMC_DCTRL_DBLOCKSIZE_0: u32 = 0x00000010;
const SDMMC_DCTRL_DBLOCKSIZE_1: u32 = 0x00000020;
const SDMMC_DCTRL_DBLOCKSIZE_2: u32 = 0x00000040;
const SDMMC_DCTRL_DBLOCKSIZE_3: u32 = 0x00000080;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DataBlockSize {
    B1 = 0,
    B2 = SDMMC_DCTRL_DBLOCKSIZE_0,
    B4 = SDMMC_DCTRL_DBLOCKSIZE_1,
    B8 = (SDMMC_DCTRL_DBLOCKSIZE_0 | SDMMC_DCTRL_DBLOCKSIZE_1),
    B16 = SDMMC_DCTRL_DBLOCKSIZE_2,
    B32 = (SDMMC_DCTRL_DBLOCKSIZE_0 | SDMMC_DCTRL_DBLOCKSIZE_2),
    B64 = (SDMMC_DCTRL_DBLOCKSIZE_1 | SDMMC_DCTRL_DBLOCKSIZE_2),
    B128 = (SDMMC_DCTRL_DBLOCKSIZE_0 | SDMMC_DCTRL_DBLOCKSIZE_1 | SDMMC_DCTRL_DBLOCKSIZE_2),
    B256 = SDMMC_DCTRL_DBLOCKSIZE_3,
    B512 = (SDMMC_DCTRL_DBLOCKSIZE_0 | SDMMC_DCTRL_DBLOCKSIZE_3),
    B1024 = (SDMMC_DCTRL_DBLOCKSIZE_1 | SDMMC_DCTRL_DBLOCKSIZE_3),
    B2048 = (SDMMC_DCTRL_DBLOCKSIZE_0 | SDMMC_DCTRL_DBLOCKSIZE_1 | SDMMC_DCTRL_DBLOCKSIZE_3),
    B4096 = (SDMMC_DCTRL_DBLOCKSIZE_2 | SDMMC_DCTRL_DBLOCKSIZE_3),
    B8192 = (SDMMC_DCTRL_DBLOCKSIZE_0 | SDMMC_DCTRL_DBLOCKSIZE_2 | SDMMC_DCTRL_DBLOCKSIZE_3),
    B16384 = (SDMMC_DCTRL_DBLOCKSIZE_1 | SDMMC_DCTRL_DBLOCKSIZE_2 | SDMMC_DCTRL_DBLOCKSIZE_3),
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum TransferDir {
    ToCard = 0,
    #[doc(alias = "SDMMC_DCTRL_DTDIR")]
    ToSdMmc = 0x00000002,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum TransferMode {
    Block = 0,
    #[doc(alias = "SDMMC_DCTRL_DTMODE_1")]
    Stream = 0x00000008,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DpsmState {
    Disable = 0,
    #[doc(alias = "SDMMC_DCTRL_DTEN")]
    Enable = 0x00000001,
}

pub struct DataInit {
    /// Data timeout period in card bus clock periods
    pub data_time_out: u32,
    /// Number of bytes to be transfered
    pub data_len: u32,

    pub data_block_size: DataBlockSize,
    pub transfer_dir: TransferDir,
    pub transfer_mode: TransferMode,
    pub dpsm: DpsmState,
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Error: u32 {
        ///  No error
        const NONE = 0x00000000;
        ///  Command response received (but CRC check failed)
        const CMD_CRC_FAIL = 0x00000001;
        ///  Data block sent/received (CRC check failed)
        const DATA_CRC_FAIL = 0x00000002;
        ///  Command response timeout
        const CMD_RSP_TIMEOUT = 0x00000004;
        ///  Data timeout
        const DATA_TIMEOUT = 0x00000008;
        ///  Transmit FIFO underrun
        const TX_UNDERRUN = 0x00000010;
        ///  Receive FIFO overrun
        const RX_OVERRUN = 0x00000020;
        ///  Misaligned address
        const ADDR_MISALIGNED = 0x00000040;
        ///  Transferred block length is not allowed for the card or the number of transferred bytes does not match the block length
        const BLOCK_LEN_ERR = 0x00000080;
        ///  An error in the sequence of erase command occurs
        const ERASE_SEQ_ERR = 0x00000100;
        ///  An invalid selection for erase groups
        const BAD_ERASE_PARAM = 0x00000200;
        ///  Attempt to program a write protect block
        const WRITE_PROT_VIOLATION = 0x00000400;
        ///  Sequence or password error has been detected in unlock command or if there was an attempt to access a locked card
        const LOCK_UNLOCK_FAILED = 0x00000800;
        ///  CRC check of the previous command failed
        const COM_CRC_FAILED = 0x00001000;
        ///  Command is not legal for the card state
        const ILLEGAL_CMD = 0x00002000;
        ///  Card internal ECC was applied but failed to correct the data
        const CARD_ECC_FAILED = 0x00004000;
        ///  Internal card controller error
        const CC_ERR = 0x00008000;
        ///  General or unknown error
        const GENERAL_UNKNOWN_ERR = 0x00010000;
        ///  The card could not sustain data reading in stream rmode
        const STREAM_READ_UNDERRUN = 0x00020000;
        ///  The card could not sustain data programming in stream mode
        const STREAM_WRITE_OVERRUN = 0x00040000;
        ///  CID/CSD overwrite error
        const CID_CSD_OVERWRITE = 0x00080000;
        ///  Only partial address space was erased
        const WP_ERASE_SKIP = 0x00100000;
        ///  Command has been executed without using internal ECC
        const CARD_ECC_DISABLED = 0x00200000;
        ///  Erase sequence was cleared before executing because an out of erase sequence command was received
        const ERASE_RESET = 0x00400000;
        ///  Error in sequence of authentication
        const AKE_SEQ_ERR = 0x00800000;
        ///  Error in case of invalid voltage range
        const INVALID_VOLTRANGE = 0x01000000;
        ///  Error when addressed block is out of range
        const ADDR_OUTOF_RANGE = 0x02000000;
        ///  Error when command request is not applicable
        const REQUEST_NOT_APPLICABLE = 0x04000000;
        ///  the used parameter is not valid
        const INVALID_PARAMETER = 0x08000000;
        ///  Error when feature is not insupported
        const UNSUPPORTED_FEATURE = 0x10000000;
        ///  Error when transfer process is busy
        const BUSY = 0x20000000;
        ///  Error while DMA transfer
        const DMA = 0x40000000;
        ///  Timeout error
        const TIMEOUT = 0x80000000;
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Command {
    ///  Resets the SD memory card.                                                               
    GoIdleState = 0,
    ///  Sends host capacity support information and activates the card's initialization process.
    SendOpCond = 1,
    ///  Asks any card connected to the host to send the CID numbers on the CMD line.             
    AllSendCid = 2,
    ///  Asks the card to publish a new relative address (RCA).                                   
    SetRelAddr = 3,
    ///  Programs the DSR of all cards.                                                           
    SetDsr = 4,
    ///  Sends host capacity support information (HCS) and asks the accessed card to send its operating condition register (OCR) content in the response on the CMD line.
    SdmmcSenOpCond = 5,
    ///  Checks switchable function (mode 0) and switch card function (mode 1).                   
    HsSwitch = 6,
    ///  Selects the card by its own relative address and gets deselected by any other address    
    SelDeselCard = 7,
    ///  Sends SD Memory Card interface condition, which includes host supply voltage information  and asks the card whether card supports voltage.                      
    HsSendExtCsd = 8,
    ///  Addressed card sends its card specific data (CSD) on the CMD line.                       
    SendCsd = 9,
    ///  Addressed card sends its card identification (CID) on the CMD line.                      
    SendCid = 10,
    ///  SD card Voltage switch to 1.8V mode.                                                     
    VoltageSwitch = 11,
    ///  Forces the card to stop transmission.                                                    
    StopTransmission = 12,
    ///  Addressed card sends its status register.                                                
    SendStatus = 13,
    ///  Reserved                                                                                 
    HsBustestRead = 14,
    ///  Sends an addressed card into the inactive state.                                         
    GoInactiveState = 15,
    ///  Sets the block length (in bytes for SDSC) for all following block commands (read, write, lock). Default block length is fixed to 512 Bytes. Not effective        
    /// for SDHS and SDXC.                                                                       
    SetBlockLen = 16,
    ///  Reads single block of size selected by SET_BLOCKLEN in case of SDSC, and a block of fixed 512 bytes in case of SDHC and SDXC.                                    
    ReadSingleBlock = 17,
    ///  Continuously transfers data blocks from card to host until interrupted by  STOP_TRANSMISSION command.                                                            
    ReadMultBlock = 18,
    ///  64 bytes tuning pattern is sent for SDR50 and SDR104.                                    
    HsBustestWrite = 19,
    ///  Speed class control command.                                                             
    WriteDatUntilStop = 20,
    ///  Specify block count for CMD18 and CMD25.                                                 
    SetBlockCount = 23,
    ///  Writes single block of size selected by SET_BLOCKLEN in case of SDSC, and a block of fixed 512 bytes in case of SDHC and SDXC.                                   
    WriteSingleBlock = 24,
    ///  Continuously writes blocks of data until a STOP_TRANSMISSION follows.                    
    WriteMultBlock = 25,
    ///  Reserved for manufacturers.                                                              
    ProgCid = 26,
    ///  Programming of the programmable bits of the CSD.                                         
    ProgCsd = 27,
    ///  Sets the write protection bit of the addressed group.                                    
    SetWriteProt = 28,
    ///  Clears the write protection bit of the addressed group.                                  
    ClrWriteProt = 29,
    ///  Asks the card to send the status of the write protection bits.                           
    SendWriteProt = 30,
    ///  Sets the address of the first write block to be erased. (For SD card only).              
    SdEraseGrpStart = 32,
    ///  Sets the address of the last write block of the continuous range to be erased.           
    SdEraseGrpEnd = 33,
    ///  Sets the address of the first write block to be erased. Reserved for each command system set by switch function command (CMD6).                                  
    EraseGrpStart = 35,
    ///  Sets the address of the last write block of the continuous range to be erased. Reserved for each command system set by switch function command (CMD6).           
    EraseGrpEnd = 36,
    ///  Reserved for SD security applications.                                                   
    Erase = 38,
    ///  SD card doesn't support it (Reserved).                                                   
    FastIo = 39,
    ///  SD card doesn't support it (Reserved).                                                   
    GoIrqState = 40,
    ///  Sets/resets the password or lock/unlock the card. The size of the data block is set by the SET_BLOCK_LEN command.                                                
    LockUnlock = 42,
    ///  Indicates to the card that the next command is an application specific command rather than a standard command.                                                   
    AppCmd = 55,
    ///  Used either to transfer a data block to the card or to get a data block from the card for general purpose/application specific commands.                         
    GenCmd = 56,
    ///  No command                                                                               
    NoCmd = 64,
}

/// SD Card Specific security commands.
/// [`Command::AppCommand`][] should be sent before sending these commands.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum SdCardCommand {
    ///  (ACMD6) Defines the data bus width to be used for data transfer. The allowed data bus widths are given in SCR register.                                                   
    AppSdSetBuswidth = 6,
    ///  (ACMD13) Sends the SD status.                                                            
    SdAppStatus = 13,
    ///  (ACMD22) Sends the number of the written (without errors) write blocks. Responds with 32bit+CRC data block.                                                               
    SdAppSendNumWriteBlocks = 22,
    ///  (ACMD41) Sends host capacity support information (HCS) and asks the accessed card to send its operating condition register (OCR) content in the response on the CMD line.
    SdAppOpCond = 41,
    ///  (ACMD42) Connect/Disconnect the 50 KOhm pull-up resistor on CD/DAT3 (pin 1) of the card  
    SdAppSetClrCardDetect = 42,
    ///  Reads the SD Configuration Register (SCR).                                               
    SdAppSendScr = 51,
    ///  For SD I/O card only, reserved for security specification.                               
    SdmmcRwDirect = 52,
    ///  For SD I/O card only, reserved for security specification.                               
    SdmmcRwExtended = 53,
}

/// Error card status R1 (OCR register)
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorcardStatus {
    AddrOutOfRange = 0x80000000,
    AddrMisaligned = 0x40000000,
    BlockLenErr = 0x20000000,
    EraseSeqErr = 0x10000000,
    BadEraseParam = 0x08000000,
    WriteProtViolation = 0x04000000,
    LockUnlockFailed = 0x01000000,
    ComCrcFailed = 0x00800000,
    IllegalCmd = 0x00400000,
    CardEccFailed = 0x00200000,
    CcError = 0x00100000,
    GeneralUnknownError = 0x00080000,
    StreamReadUnderrun = 0x00040000,
    StreamWriteOverrun = 0x00020000,
    CidCsdOverwrite = 0x00010000,
    WpEraseSkip = 0x00008000,
    CardEccDisabled = 0x00004000,
    EraseReset = 0x00002000,
    AkeSeqError = 0x00000008,
    Errorbits = 0xFDFFE008,
}

/// Masks for the R6 Response
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorCardR6Response {
    GeneralUnknownError = 0x00002000,
    IllegalCmd = 0x00004000,
    ComCrcFailed = 0x00008000,
}

pub struct Disabled;
pub struct Enabled;

pub struct SdMmcMaster<P, S> {
    peripheral: P,
    _state: PhantomData<S>,
}

impl<P: SdMmc> SdMmcMaster<P, Disabled> {
    pub fn new(peripheral: P) -> Self {
        Self {
            peripheral,
            _state: PhantomData,
        }
    }

    pub fn enable(self, init: SdMMCInit) -> SdMmcMaster<P, Enabled> {
        let tmpreg = init.clock_edge as u32
            | init.clock_power_save as u32
            | init.bus_wide as u32
            | init.hardware_flow_control as u32
            | init.clock_div as u32;
        self.peripheral.clkcr().write(|w| unsafe { w.bits(tmpreg) });
        SdMmcMaster {
            peripheral: self.peripheral,
            _state: PhantomData,
        }
    }
}

enum_u!(
    #[repr(u8)]
    #[derive(Clone, Copy, Debug)]
    pub enum PowerCtrl {
        Off = 0x0,
        On = 0x3,
        Cycle = 0x2,
    }
);

enum_u!(
    #[repr(u32)]
    /// Response registers
    #[derive(Clone, Copy, Debug)]
    pub enum Resp {
        Resp1 = 0x0,
        Resp2 = 0x4,
        Resp3 = 0x8,
        Resp4 = 0xC,
    }
);

const CMD_TIMEOUT: u32 = 5000;

impl<P: SdMmc> SdMmcMaster<P, Enabled> {
    pub fn read_fifo(&mut self) -> u32 {
        // The C hal only reads/writes the 0x80 register for FIFO
        self.peripheral.fifor0().read().fifodata().bits()
    }

    pub fn write_fifo(&mut self, value: u32) {
        self.peripheral
            .fifor0()
            .write(|w| unsafe { w.fifodata().bits(value) });
    }

    pub fn power_on(&mut self) {
        self.peripheral
            .power()
            .write(|w| unsafe { w.pwrctrl().bits(PowerCtrl::On as _) });
    }
    pub fn power_off(&mut self) {
        self.peripheral
            .power()
            .write(|w| unsafe { w.pwrctrl().bits(PowerCtrl::Off as _) });
    }
    pub fn power_cycle(&mut self) {
        self.peripheral
            .power()
            .write(|w| unsafe { w.pwrctrl().bits(PowerCtrl::Cycle as _) });
    }

    pub fn power(&self) -> PowerCtrl {
        self.peripheral
            .power()
            .read()
            .pwrctrl()
            .bits()
            .try_into()
            .expect("All power values to be supported by the enum")
    }

    pub fn send_command(&mut self, command: SdMmcCmdInit) {
        let tmpreg = command.cmd_index as u32
            | command.response as u32
            | command.wait_for_interrupt as u32
            | command.cpsm as u32;

        self.peripheral.cmdr().write(|w| unsafe { w.bits(tmpreg) });
    }

    pub fn command_response(&mut self) -> u8 {
        self.peripheral.respcmdr().read().respcmd().bits()
    }

    pub fn get_response(&mut self, resp: Resp) -> u32 {
        match resp {
            Resp::Resp1 => self.peripheral.resp1r().read().bits(),
            Resp::Resp2 => self.peripheral.resp2r().read().bits(),
            Resp::Resp3 => self.peripheral.resp3r().read().bits(),
            Resp::Resp4 => self.peripheral.resp4r().read().bits(),
        }
    }

    pub fn data_counter(&mut self) -> u32 {
        self.peripheral.dcntr().read().bits()
    }

    /// Why is this the same as read_fifo?
    pub fn fifo_count(&mut self) -> u32 {
        // The C hal only reads/writes the 0x80 register for FIFO
        self.peripheral.fifor0().read().fifodata().bits()
    }

    pub fn set_read_wait_mode(&mut self, read_wait_mode: ReadWaitMode) {
        self.peripheral
            .dctrl()
            .write(|w| w.rwmod().bit(read_wait_mode.bit()));
    }

    /// Checks for error conditions for R1 response.
    pub fn get_cmd_resp1(&mut self, cmd: Command, timeout: u32) -> Error {
        // TODO: get real freq
        let system_freq = 64_000_000;
        let mut count = system_freq / 8 / 1000;
        loop {
            count -= 1;
            if count == 0 {
                return Error::TIMEOUT;
            }

            let star = self.peripheral.star().read();
            if !(star.ccrcfail().bit()
                | star.cmdrend().bit()
                | star.ctimeout().bit()
                | star.busyd0end().bit()
                | !star.cpsmact().bit())
            {
                break;
            }
        }
        todo!()
    }

    pub fn cmd_block_len(&mut self, block_size: u32) -> Error {
        self.send_command(SdMmcCmdInit {
            argument: block_size,
            cmd_index: Command::SetBlockLen,
            response: Response::Short,
            wait_for_interrupt: WaitForInterruptState::No,
            cpsm: CpsmState::Enable,
        });

        self.get_cmd_resp1(Command::SetBlockLen, CMD_TIMEOUT)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ReadWaitMode {
    /// Read Wait control using SDMMC_DATA2
    Data2,
    /// Read Wait control by stopping SDMMCCLK
    Clk,
}

impl ReadWaitMode {
    fn bit(self) -> bool {
        matches!(self, Self::Clk)
    }
}
