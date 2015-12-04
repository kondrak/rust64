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

pub struct CIA
{
    mem_ref: Option<memory::MemShared>,
    cpu_ref: Option<cpu::CPUShared>,
    vic_ref: Option<vic::VICShared>,

    ta_state: TimerState,  // timer A state
    tb_state: TimerState,  // timer B state
    ta: u16,
    tb: u16,
    ta_cnt_phi2: bool,
    tb_cnt_phi2: bool,
    tb_cnt_ta: bool,
    ta_underflow: bool,
    icr: u8,
    ta_irq_next_cycle: bool,
    tb_irq_next_cycle: bool,
    latch_a: u16,
    latch_b: u16,
    cra: u8,
    crb: u8,
    new_cra: u8,
    new_crb: u8,
    has_new_cra: bool,
    has_new_crb: bool,
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
            
            ta_state: TimerState::STOP,
            tb_state: TimerState::STOP,
            ta: 0,
            tb: 0,
            ta_cnt_phi2: false,
            tb_cnt_phi2: false,
            tb_cnt_ta: false,
            ta_underflow: false,
            icr: 0,
            ta_irq_next_cycle: false,
            tb_irq_next_cycle: false,
            latch_a: 0,
            latch_b: 0,
            cra: 0,
            crb: 0,
            new_cra: 0,
            new_crb: 0,
            has_new_cra: false,
            has_new_crb: false,
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
        self.ta_state = TimerState::STOP;
        self.tb_state = TimerState::STOP;
        self.ta = 0;
        self.tb = 0;
        self.ta_cnt_phi2 = false;
        self.tb_cnt_phi2 = false;
        self.tb_cnt_ta = false;
        self.ta_underflow = false;
        self.icr = 0;
        self.ta_irq_next_cycle = false;
        self.tb_irq_next_cycle = false;
        self.latch_a = 0;
        self.latch_b = 0;
        self.cra = 0;
        self.crb = 0;
        self.new_cra = 0;
        self.new_crb = 0;
        self.has_new_cra = false;
        self.has_new_crb = false;
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
        self.ta_underflow = false;
        self.update_timer_a();
        self.update_timer_b();        
    }

    fn update_timer_a(&mut self)
    {
        match self.ta_state
        {
            TimerState::STOP => (),            
            TimerState::WAIT_COUNT => {
                self.ta_state = TimerState::COUNT;
            },
            TimerState::LOAD_STOP => {
                self.ta_state = TimerState::STOP;
                self.ta = self.latch_a;
            },
            TimerState::LOAD_COUNT => {
                self.ta_state = TimerState::COUNT;
                self.ta = self.latch_a;
            },
            TimerState::LOAD_WAIT_COUNT => {
                self.ta_state = TimerState::WAIT_COUNT;

                if self.ta == 1
                {
                    self.ta_irq();
                }
                else
                {
                    self.ta = self.latch_a;
                }
            },
            TimerState::COUNT => {
                self.ta_count();
            },
            TimerState::COUNT_STOP => {
                self.ta_state = TimerState::STOP;
                self.ta_count();
            }
        }

        self.ta_idle();
    }

    fn ta_idle(&mut self)
    {
        if self.has_new_cra
        {
            match self.ta_state
            {
                TimerState::STOP | TimerState::LOAD_STOP =>
                {
                    if (self.new_cra & 1) != 0
                    {
                        if (self.new_cra & 0x10) != 0
                        {
                            self.ta_state = TimerState::LOAD_WAIT_COUNT;
                        }
                        else
                        {
                            self.ta_state = TimerState::WAIT_COUNT;
                        }
                    }
                    else
                    {
                        if (self.new_cra & 0x10) != 0
                        {
                            self.ta_state = TimerState::LOAD_STOP;
                        }
                    }
                      
                },
                TimerState::WAIT_COUNT | TimerState::LOAD_COUNT =>
                {
                    if (self.new_cra & 1) != 0
                    {
                        if (self.new_cra & 8) != 0
                        {
                            self.new_cra &= 0xFE;
                            self.ta_state = TimerState::STOP;
                        }
                        else
                        {
                            if (self.new_cra & 0x10) != 0
                            {
                                self.ta_state = TimerState::LOAD_WAIT_COUNT;
                            }
                        }
                    }
                    else
                    {
                        self.ta_state = TimerState::STOP;
                    }
                },
                TimerState::COUNT =>
                {
                    if (self.new_cra & 1) != 0
                    {
                        if (self.new_cra & 0x10) != 0
                        {
                            self.ta_state = TimerState::LOAD_WAIT_COUNT;
                        }
                    }
                    else
                    {
                        if (self.new_cra & 0x10) != 0
                        {
                            self.ta_state = TimerState::LOAD_STOP;
                        }
                        else
                        {
                            self.ta_state = TimerState::COUNT_STOP;
                        }
                    }
                },
                _ => (),
            }

            self.cra = self.new_cra & 0xEF;
            self.has_new_cra = false;
        }
    }

    fn ta_irq(&mut self)
    {
        self.ta = self.latch_a;
        self.ta_irq_next_cycle = true;
        self.icr |= 1;

        if (self.cra & 8) != 0
        {
            self.cra &= 0xFE;
            self.new_cra &= 0xFE;
            self.ta_state = TimerState::LOAD_STOP;
        }
        else
        {
            self.ta_state = TimerState::LOAD_COUNT;
        }
    }

    
    fn ta_count(&mut self)
    {
        if self.ta_cnt_phi2
        {
            let curr_ta = self.ta;
            self.ta -= 1;
            if (curr_ta == 0) || (self.ta == 0)
            {
                match self.ta_state
                {
                    TimerState::STOP => (),
                    _ => self.ta_irq(),
                }

                self.ta_underflow = true;
            }
        }
    }     

    fn update_timer_b(&mut self)
    {
        match self.tb_state
        {
            TimerState::STOP => (),
            TimerState::WAIT_COUNT => {
                self.tb_state = TimerState::COUNT;
            },
            TimerState::LOAD_STOP => {
                self.tb_state = TimerState::COUNT;
                self.tb = self.latch_b;
            },
            TimerState::LOAD_COUNT => {
                self.tb_state = TimerState::COUNT;
                self.tb = self.latch_b;
            },
            TimerState::LOAD_WAIT_COUNT => {
                self.tb_state = TimerState::WAIT_COUNT;

                if self.tb == 1
                {
                    self.tb_irq();
                }
                else
                {
                    self.tb = self.latch_b;
                }
            }
            TimerState::COUNT => {
                self.tb_count();
            },
            TimerState::COUNT_STOP => {
                self.tb_state = TimerState::STOP;
                self.tb_count();
            }
        }

        self.tb_idle();
    }    

    fn tb_idle(&mut self)
    {
        if self.has_new_crb
        {
            match self.tb_state
            {
                TimerState::STOP | TimerState::LOAD_STOP =>
                {
                    if (self.new_crb & 1) != 0
                    {
                        if (self.new_crb & 0x10) != 0
                        {
                            self.tb_state = TimerState::LOAD_WAIT_COUNT;
                        }
                        else
                        {
                            self.tb_state = TimerState::WAIT_COUNT;
                        }
                    }
                    else
                    {
                        if (self.new_crb & 0x10) != 0
                        {
                            self.tb_state = TimerState::LOAD_STOP;
                        }
                    }
                      
                },
                TimerState::WAIT_COUNT | TimerState::LOAD_COUNT =>
                {
                    if (self.new_crb & 1) != 0
                    {
                        if (self.new_crb & 8) != 0
                        {
                            self.new_crb &= 0xFE;
                            self.tb_state = TimerState::STOP;
                        }
                        else
                        {
                            if (self.new_crb & 0x10) != 0
                            {
                                self.tb_state = TimerState::LOAD_WAIT_COUNT;
                            }
                        }
                    }
                    else
                    {
                        self.tb_state = TimerState::STOP;
                    }
                },
                TimerState::COUNT =>
                {
                    if (self.new_crb & 1) != 0
                    {
                        if (self.new_crb & 0x10) != 0
                        {
                            self.tb_state = TimerState::LOAD_WAIT_COUNT;
                        }
                    }
                    else
                    {
                        if (self.new_crb & 0x10) != 0
                        {
                            self.tb_state = TimerState::LOAD_STOP;
                        }
                        else
                        {
                            self.tb_state = TimerState::COUNT_STOP;
                        }
                    }
                },
                _ => (),
            }

            self.crb = self.new_crb & 0xEF;
            self.has_new_crb = false;
        }
    }  
    
    fn tb_irq(&mut self)
    {
        self.tb = self.latch_b;
        self.tb_irq_next_cycle = true;
        self.icr |= 2;

        if (self.crb & 8) != 0
        {
            self.crb &= 0xFE;
            self.new_crb &= 0xFE;
            self.tb_state = TimerState::LOAD_STOP;
        }
        else
        {
            self.tb_state = TimerState::LOAD_COUNT;
        }
    }

    fn tb_count(&mut self)
    {
        if self.tb_cnt_phi2 || (self.tb_cnt_ta && self.ta_underflow)
        {
            let curr_tb = self.tb;
            self.tb -= 1;
            if (curr_tb == 0) || (self.tb == 0)
            {
                match self.tb_state
                {
                    TimerState::STOP => (),
                    _ => self.tb_irq(),
                }
            }
        }
    }
}
