// SID
extern crate rand;
use c64::memory;
use c64::cpu;
use c64::sid_tables::*;
use std::cell::RefCell;
use std::rc::Rc;

pub type SIDShared = Rc<RefCell<SID>>;

const SAMPLE_FREQ: u32 = 44100;  // output frequency
const SID_FREQ:    u32 = 985248; // SID frequency in Hz
const CALC_FREQ:   u32 = 50;     // frequency of calculating new buffer data (50Hz)
pub const SID_CYCLES:  u32 = SID_FREQ/SAMPLE_FREQ;  // SID clocks/sample frame
const NUM_SAMPLES: usize = 624; // size of buffer for sampled voice

enum WaveForm
{
    None,
    Triangle,
    Saw,
    TriSaw,
    Rectangle,
    TriRect,
    SawRect,
    TriSawRect,
    Noise
}

enum VoiceState
{
    Idle,
    Attack,
    Decay,
    Release
}

enum FilterType
{
    None,
    Lowpass,
    Bandpass,
    LowBandpass,
    Highpass,
    Notch,
    HighBandpass,
    All
}

struct SIDVoice
{
    wave: WaveForm,
    state: VoiceState,
    modulator: u8,   // number of voice that modulates this voice
    modulatee: u8,   // number of voice that this voice modulates
    wf_cnt: u32,     // waveform counter
    wf_add: u32,     // value to add to wf_cnt each frame
    freq: u16,
    pw_val: u16,     // pulse-width value
    attack_add: u32,
    decay_sub: u32,
    release_sub: u32,
    sustain_level: u32,
    level: u32,
    noise: u32,
    gate: bool,
    ring: bool,
    test: bool,
    filter: bool,
    sync: bool,
    mute: bool     // only voice 3 can be muted
}

impl SIDVoice
{
    fn new() -> SIDVoice
    {
        SIDVoice
        {
            wave: WaveForm::None,
            state: VoiceState::Idle,
            modulator: 0,
            modulatee: 0,
            wf_cnt: 0,
            wf_add: 0,
            freq: 0,
            pw_val: 0,
            attack_add: EG_TABLE[0],
            decay_sub: EG_TABLE[0],
            release_sub: EG_TABLE[0],
            sustain_level: 0,
            level: 0,
            noise: 0,
            gate: false,
            ring: false,
            test: false,
            filter: false,
            sync: false,
            mute: false
        }
    }
}

impl SIDVoice
{
    fn reset(&mut self)
    {
        self.wave = WaveForm::None;
        self.state = VoiceState::Idle;
        self.wf_cnt = 0;
        self.wf_add = 0;
        self.freq = 0;
        self.pw_val = 0;
        self.attack_add = EG_TABLE[0];
        self.decay_sub = EG_TABLE[0];
        self.release_sub = EG_TABLE[0];
        self.sustain_level = 0;
        self.level = 0;
        self.noise = 0;
        self.gate = false;
        self.ring = false;
        self.test = false;
        self.filter = false;
        self.sync = false;
        self.mute = false;
    }
}

pub struct SID
{
    mem_ref: Option<memory::MemShared>,
    last_sid_byte: u8,
    voices: Vec<SIDVoice>,
    rng: u32,
}

impl SID
{
    pub fn new_shared() -> SIDShared
    {
        let sid_shared = Rc::new(RefCell::new(SID
        {
            mem_ref: None,
            last_sid_byte: 0,
            voices: vec![SIDVoice::new(), SIDVoice::new(), SIDVoice::new()],
            rng: 1
        }));

        // calculate triangle table values
        unsafe
        {
            for i in 0..8192
            {
                let val = ((i << 4) | (i >> 8)) as u16;
                TRI_TABLE[i] = val;
                TRI_TABLE[8191 - i] = val;
            }
        }

        sid_shared.borrow_mut().voices[0].modulator = 2;
        sid_shared.borrow_mut().voices[0].modulatee = 1;
        sid_shared.borrow_mut().voices[1].modulator = 0;
        sid_shared.borrow_mut().voices[1].modulatee = 2;
        sid_shared.borrow_mut().voices[2].modulator = 1;
        sid_shared.borrow_mut().voices[2].modulatee = 0;
        
        sid_shared
    }

    pub fn set_references(&mut self, memref: memory::MemShared)
    {
        self.mem_ref = Some(memref);
    }

    pub fn reset(&mut self)
    {
        // TODO
        self.last_sid_byte = 0;

        for i in 0..self.voices.len()
        {
            self.voices[i].reset();
        }
    }

    fn lowpass_resonance(&self, f: f64) -> f64
    {
        let f2 = f * f;
        let f3 = f2 * f;
        let f4 = f3 * f;
        227.755 - f - 1.7653 * f - 0.0176385 * f2 + 0.00333484 * f3 - 9.05683E-6 * f4
    }

    fn highpass_resonance(&self, f: f64) -> f64
    {
        let f2 = f * f;
        let f3 = f2 * f;
        366.374 - 14.0052 * f + 0.603212 * f2 - 0.000880196 * f3
    }    

    fn get_rand(&mut self) -> u8
    {
        self.rng = self.rng * 1103515245 + 12345;
        (self.rng >> 16) as u8
    }
    
    pub fn read_register(&mut self, addr: u16) -> u8
    {
        // most SID registers are write-only. The write to IO RAM is performed
        // so that the debugger can print out the value fetched by the CPU
        match addr
        {
            0xD419...0xD41A => {
                self.last_sid_byte = 0;
                let rval = 0xFF;
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, rval);
                rval
            },
            0xD41B...0xD41C => {
                self.last_sid_byte = 0;
                let rval = rand::random::<u8>();
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, rval);
                rval
            },
            0xD420...0xD7FF => self.read_register(0xD400 + (addr % 0x0020)),
            _               =>  {
                let rval = self.last_sid_byte;
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, rval);
                rval
            }
        }
    }
    
    pub fn write_register(&mut self, addr: u16, value: u8, on_sid_write: &mut cpu::Callback)
    {
        self.last_sid_byte = value;
        match addr
        {
            // TODO
            0xD420...0xD7FF => self.write_register(0xD400 + (addr % 0x0020), value, on_sid_write),
            _               => as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, value)
        }

        *on_sid_write = cpu::Callback::None;
    }
    
    pub fn update(&mut self)
    {
        // TODO
    }
}
