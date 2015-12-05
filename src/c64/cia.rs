// CIA
#![allow(dead_code)]
use c64::cpu;
use c64::vic;
use c64::memory;
use std::rc::Rc;
use std::cell::RefCell;

pub type CIAShared = Rc<RefCell<CIA>>;

pub enum CIACallbackAction
{
    CIA_NONE,
}

enum TimerState
{
    STOP,
    WAIT_COUNT,
    LOAD_STOP,
    LOAD_COUNT,
    LOAD_WAIT_COUNT,
    COUNT,
    COUNT_STOP
}
// Struct for CIA timer A/B
struct CIATimer
{
    state: TimerState, // current state of the timer
    is_ta: bool, // is this timer A?
    value: u16,  // timer value (TA/TB)
    latch: u16,  // timer latch
    ctrl: u8,    // control timer (CRA/CRB)
    new_ctrl: u8,
    has_new_ctrl: bool,
    is_cnt_phi2: bool,      // timer is counting phi2
    irq_next_cycle: bool,   // perform timer interrupt next cycle
    underflow: bool,        // timer underflowed
    cnt_ta_underflow: bool, // timer is counting underflows of Timer A 
}

impl CIATimer
{
    pub fn new(is_ta: bool) -> CIATimer
    {
        CIATimer
        {
            state: TimerState::STOP,
            is_ta: is_ta,
            value: 0xFFFF,
            latch: 1,
            ctrl:  0,
            new_ctrl: 0,
            has_new_ctrl: false,
            is_cnt_phi2:  false,
            irq_next_cycle:   false,
            underflow:        false,
            cnt_ta_underflow: false,
        }
    }
    pub fn reset(&mut self)
    {
        self.state    = TimerState::STOP;
        self.value    = 0xFFFF;
        self.latch    = 1;
        self.ctrl     = 0;
        self.new_ctrl = 0;
        self.has_new_ctrl     = false;
        self.is_cnt_phi2      = false;
        self.irq_next_cycle   = false;
        self.underflow        = false;
        self.cnt_ta_underflow = false;
    }
    
    pub fn update(&mut self, cia_icr: &mut u8, ta_underflow: bool)
    {
        match self.state
        {
            TimerState::STOP => (),
            TimerState::WAIT_COUNT => {
                self.state = TimerState::COUNT;
            },
            TimerState::LOAD_STOP => {
                self.state = TimerState::STOP;
                self.value = self.latch;
            },
            TimerState::LOAD_COUNT => {
                self.state = TimerState::COUNT;
                self.value = self.latch;
            },
            TimerState::LOAD_WAIT_COUNT => {
                self.state = TimerState::WAIT_COUNT;

                if self.value == 1
                {
                    self.irq(cia_icr);
                }
                else
                {
                    self.value = self.latch;
                }
            }
            TimerState::COUNT => {
                self.count(cia_icr, ta_underflow);
            },
            TimerState::COUNT_STOP => {
                self.state = TimerState::STOP;
                self.count(cia_icr, ta_underflow);
            }
        }

        self.idle();        
    }
    
    pub fn idle(&mut self)
    {
        if self.has_new_ctrl
        {
            match self.state
            {
                TimerState::STOP | TimerState::LOAD_STOP =>
                {
                    if (self.new_ctrl & 1) != 0
                    {
                        if (self.new_ctrl & 0x10) != 0
                        {
                            self.state = TimerState::LOAD_WAIT_COUNT;
                        }
                        else
                        {
                            self.state = TimerState::WAIT_COUNT;
                        }
                    }
                    else
                    {
                        if (self.new_ctrl & 0x10) != 0
                        {
                            self.state = TimerState::LOAD_STOP;
                        }
                    }
                      
                },
                TimerState::WAIT_COUNT | TimerState::LOAD_COUNT =>
                {
                    if (self.new_ctrl & 1) != 0
                    {
                        if (self.new_ctrl & 8) != 0
                        {
                            self.new_ctrl &= 0xFE;
                            self.state = TimerState::STOP;
                        }
                        else
                        {
                            if (self.new_ctrl & 0x10) != 0
                            {
                                self.state = TimerState::LOAD_WAIT_COUNT;
                            }
                        }
                    }
                    else
                    {
                        self.state = TimerState::STOP;
                    }
                },
                TimerState::COUNT =>
                {
                    if (self.new_ctrl & 1) != 0
                    {
                        if (self.new_ctrl & 0x10) != 0
                        {
                            self.state = TimerState::LOAD_WAIT_COUNT;
                        }
                    }
                    else
                    {
                        if (self.new_ctrl & 0x10) != 0
                        {
                            self.state = TimerState::LOAD_STOP;
                        }
                        else
                        {
                            self.state = TimerState::COUNT_STOP;
                        }
                    }
                },
                _ => (),
            }

            self.ctrl = self.new_ctrl & 0xEF;
            self.has_new_ctrl = false;
        }
    }
        
    pub fn irq(&mut self, cia_icr: &mut u8)
    {
        self.value = self.latch;
        self.irq_next_cycle = true;
        *cia_icr |= if self.is_ta { 1 } else { 2 };

        if (self.ctrl & 8) != 0
        {
            self.ctrl &= 0xFE;
            self.new_ctrl &= 0xFE;
            self.state = TimerState::LOAD_STOP;
        }
        else
        {
            self.state = TimerState::LOAD_COUNT;
        }
    }

    pub fn count(&mut self, cia_icr: &mut u8, ta_underflow: bool)
    {
        if self.is_cnt_phi2 || (self.cnt_ta_underflow && ta_underflow)
        {
            let curr_val = self.value;
            self.value -= 1;
            if (curr_val == 0) || (self.value == 0)
            {
                match self.state
                {
                    TimerState::STOP => (),
                    _ => self.irq(cia_icr),
                }

                self.underflow = true;
            }
        }        
    }
}

pub struct CIA
{
    mem_ref: Option<memory::MemShared>,
    cpu_ref: Option<cpu::CPUShared>,
    vic_ref: Option<vic::VICShared>,

    timer_a: CIATimer,
    timer_b: CIATimer,
    icr: u8,
}

impl CIA
{
    pub fn new_shared() -> CIAShared
    {
        Rc::new(RefCell::new(CIA
        {
            mem_ref: None,
            cpu_ref: None,
            vic_ref: None,

            timer_a: CIATimer::new(true),
            timer_b: CIATimer::new(false),
            icr: 0,
        }))
    }

    pub fn set_references(&mut self, memref: memory::MemShared, cpuref: cpu::CPUShared, vicref: vic::VICShared)
    {
        self.mem_ref = Some(memref);
        self.cpu_ref = Some(cpuref);
        self.vic_ref = Some(vicref);
    }
    
    pub fn reset(&mut self)
    {
        // TODO
        self.timer_a.reset();
        self.timer_b.reset();
        self.icr = 0;
    }

    pub fn read_register(&self, addr: u16) -> u8
    {
        // TODO
        0
    }

    pub fn write_register(&mut self, addr: u16, value: u8, on_cia_write: &mut CIACallbackAction) -> bool
    {
        // TODO
        true
    }
    
    pub fn update(&mut self)
    {
        self.timer_a.update(&mut self.icr, false);
        let ta_underflow = self.timer_a.underflow;
        self.timer_b.update(&mut self.icr, ta_underflow);
    }
}
