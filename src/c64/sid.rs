// SID
extern crate rand;
use c64::memory;
use c64::cpu;
use c64::sid_tables::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::f32;

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
    volume: u8,
    filter_type: FilterType,
    filter_freq: u8,
    filter_resonance: u8,

    // IIR filter
    iir_att: f32,
    d1: f32,
    d2: f32,
    g1: f32,
    g2: f32,
    xn1: f32,
    xn2: f32,
    yn1: f32,
    yn2: f32,
    
    voices: Vec<SIDVoice>,
    sample_buffer: [u8; NUM_SAMPLES],
    sample_idx: usize,
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
            volume: 0,
            filter_type: FilterType::None,
            filter_freq: 0,
            filter_resonance: 0,
            iir_att: 1.0,
            d1: 0.0,
            d2: 0.0,
            g1: 0.0,
            g2: 0.0,
            xn1: 0.0,
            xn2: 0.0,
            yn1: 0.0,
            yn2: 0.0,
            sample_buffer: [0; NUM_SAMPLES],
            sample_idx: 0,
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
        self.last_sid_byte = 0;

        for i in 0..self.voices.len()
        {
            self.voices[i].reset();
        }

        self.volume = 0;
        self.filter_type = FilterType::None;
        self.filter_freq = 0;
        self.filter_resonance = 0;
        self.xn1 = 0.0;
        self.xn2 = 0.0;
        self.yn1 = 0.0;
        self.yn2 = 0.0;
        self.sample_idx = 0;
        self.calculate_filter();

        for i in 0..NUM_SAMPLES
        {
            self.sample_buffer[i] = 0;
        }
    }

    fn lowpass_resonance(&self, f: f32) -> f32
    {
        let f2 = f * f;
        let f3 = f2 * f;
        let f4 = f3 * f;
        227.755 - f - 1.7653 * f - 0.0176385 * f2 + 0.00333484 * f3 - 9.05683E-6 * f4
    }

    fn highpass_resonance(&self, f: f32) -> f32
    {
        let f2 = f * f;
        let f3 = f2 * f;
        366.374 - 14.0052 * f + 0.603212 * f2 - 0.000880196 * f3
    }    

    fn calculate_filter(&mut self)
    {
        let f = self.filter_freq as f32;
        let mut resonance: f32 = 0.0;
        let mut arg: f32 = 0.0;
        
        match self.filter_type
        {
            FilterType::None => {
                self.d1 = 0.0;
                self.d2 = 0.0;
                self.g1 = 0.0;
                self.g2 = 0.0;
                self.iir_att = 1.0;
                return;
            },
            FilterType::All => {
                self.d1 = 0.0;
                self.d2 = 0.0;
                self.g1 = 0.0;
                self.g2 = 0.0;
                self.iir_att = 0.0;
                return;
            }
            FilterType::Lowpass | FilterType::LowBandpass => {
               resonance = self.lowpass_resonance(f);
            },
            _ => {
                resonance = self.highpass_resonance(f);
            }
        }

        arg = resonance / ((SAMPLE_FREQ >> 1) as f32);
        if arg > 0.99 { arg = 0.99; }
        if arg < 0.01 { arg = 0.01; }

        self.g2 = 0.55 + 1.2 * arg * arg - 1.2 * arg + resonance * 0.0133333333;
        self.g1 = -2.0 * self.g2.sqrt() * (f32::consts::PI * arg).cos();

        match self.filter_type {
            FilterType::LowBandpass | FilterType::HighBandpass => self.g2 += 0.1,
            _ => ()
        }

        if self.g1.abs() >= (self.g2 + 1.0)
        {
            if self.g1 > 0.0 { self.g1 = self.g2 + 0.99;    }
            else             { self.g1 = -(self.g2 + 0.99); }
        }

        match self.filter_type {
            FilterType::LowBandpass | FilterType::Lowpass => {
                self.d1 = 0.0;
                self.d2 = 1.0;
                self.iir_att = 0.25 * (1.0 + self.g1 + self.g2);
            },
            FilterType::HighBandpass | FilterType::Highpass => {
                self.d1 = -2.0;
                self.d2 = 1.0;
                self.iir_att = 0.25 * (1.0 - self.g1 + self.g2);
            },
            FilterType::Bandpass => {
                self.d1 = 0.0;
                self.d2 = -1.0;
                self.iir_att = 0.25 * (1.0 + self.g1 + self.g2) * (1.0 + (f32::consts::PI * arg).cos()) / (f32::consts::PI * arg).sin();
            },
            FilterType::Notch => {
                self.d1 = -2.0 * (f32::consts::PI * arg).cos();
                self.d2 = 1.0;
                self.iir_att = 0.25 * (1.0 + self.g1 + self.g2) * (1.0 + (f32::consts::PI * arg).cos()) / (f32::consts::PI * arg).sin();
            },
            _ => ()
        }
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
        let idx = self.sample_idx;
        self.sample_buffer[idx] = self.volume;
        self.sample_idx = (self.sample_idx + 1) % NUM_SAMPLES;
    }
}
