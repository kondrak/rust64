// SID chip
extern crate rand;
extern crate sdl2;

use self::sdl2::audio::{ AudioCallback, AudioSpecDesired };
use c64::memory;
use c64::sid_tables::*;
use std::cell::RefCell;
use std::f32;
use std::rc::Rc;

pub type SIDShared = Rc<RefCell<SID>>;

const SAMPLE_FREQ: u32 = 44100;  // output frequency
const SID_FREQ:    u32 = 985248; // SID frequency in Hz
pub const SID_CYCLES:  u32 = SID_FREQ / SAMPLE_FREQ;  // SID clocks/sample frame
const NUM_SAMPLES: usize = 624; // size of buffer for sampled voice


enum WaveForm {
    None,
    Triangle,
    Saw,
    TriSaw,
    Pulse,
    TriPulse,
    SawPulse,
    TriSawPulse,
    Noise
}

enum VoiceState {
    Idle,
    Attack,
    Decay,
    Release
}

#[derive(PartialEq)]
enum FilterType {
    None,
    Lowpass,
    Bandpass,
    LowBandpass,
    Highpass,
    Notch,
    HighBandpass,
    All
}


// single SID voice
struct SIDVoice {
    wave: WaveForm,
    state: VoiceState,
    modulator: usize,   // number of voice that modulates this voice
    modulatee: usize,   // number of voice that this voice modulates
    wf_cnt: u32,        // waveform counter
    wf_add: u32,        // value to add to wf_cnt each frame
    freq: u16,
    pw_val: u16,        // pulse-width value
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

impl SIDVoice {
    fn new() -> SIDVoice {
        SIDVoice {
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


    fn reset(&mut self) {
        self.wave  = WaveForm::None;
        self.state = VoiceState::Idle;
        self.wf_cnt = 0;
        self.wf_add = 0;
        self.freq   = 0;
        self.pw_val = 0;
        self.attack_add  = EG_TABLE[0];
        self.decay_sub   = EG_TABLE[0];
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


// the SID chip with associated SDL2 audio device
pub struct SID {
    mem_ref: Option<memory::MemShared>,
    audio_device: sdl2::audio::AudioDevice<SIDAudioDevice>,
}

impl SID {
    pub fn new_shared() -> SIDShared {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // mono
            samples: Some(512), // default sample size
        };
        
        Rc::new(RefCell::new(SID {
            mem_ref: None,
            audio_device: audio_subsystem.open_playback(None, &desired_spec, |spec| {
                println!("{:?}", spec);
                SIDAudioDevice::new()
                }).unwrap()
        }))
    }


    pub fn set_references(&mut self, memref: memory::MemShared) {
        self.mem_ref = Some(memref);
    }


    pub fn reset(&mut self) {
        let mut lock = self.audio_device.lock();
        (*lock).reset();
    }


    pub fn update(&mut self) {
        let mut lock = self.audio_device.lock();
        (*lock).update();
    }


    pub fn read_register(&mut self, addr: u16) -> u8 {
        let mut rval = 0;

        match addr {
            0xD419...0xD41A => {
                let mut lock = self.audio_device.lock();
                rval = (*lock).read_register(addr);
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::Io).write(addr, rval);
            },
            0xD41B...0xD41C => {
                let mut lock = self.audio_device.lock();
                rval = (*lock).read_register(addr);
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::Io).write(addr, rval);
            },
            0xD420...0xD7FF =>  { rval = self.read_register(0xD400 + (addr % 0x0020)); },
            _               =>  {
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::Io).write(addr, rval);
            }
        }

        rval
    }


    pub fn write_register(&mut self, addr: u16, value: u8) {
        let mut lock = self.audio_device.lock();
        (*lock).write_register(addr, value);
        as_ref!(self.mem_ref).get_ram_bank(memory::MemType::Io).write(addr, value);
    }


    pub fn update_audio(&mut self) {
        self.audio_device.resume();
    }
}


// SDL2 audio device along with necessary SID parameters
// this is where the actual SID calculations are being performed
struct SIDAudioDevice {
    last_sid_byte: u8,  // last byte read by the SID
    volume: u8,
    filter_type: FilterType,
    filter_freq: u8,
    filter_resonance: u8,

    // IIR filter
    iir_att: f32,
    d1:  f32,
    d2:  f32,
    g1:  f32,
    g2:  f32,
    xn1: f32,
    xn2: f32,
    yn1: f32,
    yn2: f32,
    
    voices: Vec<SIDVoice>,
    sample_buffer: [u8; NUM_SAMPLES],
    sample_idx: usize
}

impl SIDAudioDevice {
    pub fn new() -> SIDAudioDevice {
        let mut sid_audio_device = SIDAudioDevice {
            last_sid_byte: 0,
            voices: vec![SIDVoice::new(), SIDVoice::new(), SIDVoice::new()],
            volume: 0,
            filter_type: FilterType::None,
            filter_freq: 0,
            filter_resonance: 0,
            iir_att: 1.0,
            d1:  0.0,
            d2:  0.0,
            g1:  0.0,
            g2:  0.0,
            xn1: 0.0,
            xn2: 0.0,
            yn1: 0.0,
            yn2: 0.0,
            sample_buffer: [0; NUM_SAMPLES],
            sample_idx: 0
        };
       
        // calculate triangle table values
        unsafe {
            for i in 0..0x1000 {
                let val = ((i << 4) | (i >> 8)) as u16;
                TRI_TABLE[i] = val;
                TRI_TABLE[0x1FFF - i] = val;
            }
        }

        sid_audio_device.voices[0].modulator = 2;
        sid_audio_device.voices[0].modulatee = 1;
        sid_audio_device.voices[1].modulator = 0;
        sid_audio_device.voices[1].modulatee = 2;
        sid_audio_device.voices[2].modulator = 1;
        sid_audio_device.voices[2].modulatee = 0;
        
        sid_audio_device
    }


    pub fn reset(&mut self) {
        self.last_sid_byte = 0;

        for i in 0..self.voices.len() {
            self.voices[i].reset();
        }

        self.volume = 0;
        self.filter_type = FilterType::None;
        self.filter_freq = 0;
        self.filter_resonance = 0;
        self.iir_att = 1.0;
        self.d1 = 0.0;
        self.d2 = 0.0;
        self.g1 = 0.0;
        self.g2 = 0.0;
        self.xn1 = 0.0;
        self.xn2 = 0.0;
        self.yn1 = 0.0;
        self.yn2 = 0.0;
        self.sample_idx = 0;
        self.calculate_filter();

        for i in 0..NUM_SAMPLES {
            self.sample_buffer[i] = 0;
        }
    }


    pub fn update(&mut self) {
        let idx = self.sample_idx;
        self.sample_buffer[idx] = self.volume;
        self.sample_idx = (self.sample_idx + 1) % NUM_SAMPLES;
    }


    pub fn read_register(&mut self, addr: u16) -> u8 {
        // most SID registers are write-only. The write to IO RAM is performed
        // so that the debugger can print out the value fetched by the CPU
        match addr {
            0xD419...0xD41A => {
                self.last_sid_byte = 0;
                let rval = 0xFF;
                rval
            },
            0xD41B...0xD41C => {
                self.last_sid_byte = 0;
                let rval = rand::random::<u8>();
                rval
            },
            0xD420...0xD7FF => self.read_register(0xD400 + (addr % 0x0020)),
            _               =>  {
                let rval = self.last_sid_byte;
                rval
            }
        }
    }
    

    pub fn write_register(&mut self, addr: u16, value: u8) {
        self.last_sid_byte = value;

        match addr {
            0xD400 => {
                self.voices[0].freq = (self.voices[0].freq & 0xFF00) | value as u16;
                self.voices[0].wf_add = SID_CYCLES * self.voices[0].freq as u32;
            },
            0xD401 => {
                self.voices[0].freq = (self.voices[0].freq & 0x00FF) | ((value as u16) << 8);
                self.voices[0].wf_add = SID_CYCLES * self.voices[0].freq as u32;
            },
            0xD402 => {
                self.voices[0].pw_val = (self.voices[0].pw_val & 0x0F00) | value as u16;
            },
            0xD403 => {
                self.voices[0].pw_val = (self.voices[0].pw_val & 0x00FF) | (((value as u16) & 0x000F) << 8);
            },
            0xD404 => {
                self.set_control_register(0, value);
            },
            0xD405 => {
                self.voices[0].attack_add = EG_TABLE[ (value >> 4) as usize ];
                self.voices[0].decay_sub  = EG_TABLE[ (value & 0x0F) as usize ];
            },
            0xD406 => {
                self.voices[0].sustain_level = 0x111111 * (value >> 4) as u32;
                self.voices[0].release_sub   = EG_TABLE[ (value & 0x0F) as usize ];
            },
            0xD407 => {
                self.voices[1].freq = (self.voices[1].freq & 0xFF00) | value as u16;
                self.voices[1].wf_add = SID_CYCLES * self.voices[1].freq as u32;
            },
            0xD408 => {
                self.voices[1].freq = (self.voices[1].freq & 0x00FF) | ((value as u16) << 8);
                self.voices[1].wf_add = SID_CYCLES * self.voices[1].freq as u32;
            },
            0xD409 => {
                self.voices[1].pw_val = (self.voices[1].pw_val & 0x0F00) | value as u16;
            },
            0xD40A => {
                self.voices[1].pw_val = (self.voices[1].pw_val & 0x00FF) | (((value as u16) & 0x000F) << 8);
            },
            0xD40B => {
                self.set_control_register(1, value);
            },
            0xD40C => {
                self.voices[1].attack_add = EG_TABLE[ (value >> 4) as usize ];
                self.voices[1].decay_sub  = EG_TABLE[ (value & 0x0F) as usize ];
            },
            0xD40D => {
                self.voices[1].sustain_level = 0x111111 * (value >> 4) as u32;
                self.voices[1].release_sub   = EG_TABLE[ (value & 0x0F) as usize ];
            },
            0xD40E => {
                self.voices[2].freq = (self.voices[2].freq & 0xFF00) | value as u16;
                self.voices[2].wf_add = SID_CYCLES * self.voices[2].freq as u32;
            },
            0xD40F => {
                self.voices[2].freq = (self.voices[2].freq & 0x00FF) | ((value as u16) << 8);
                self.voices[2].wf_add = SID_CYCLES * self.voices[2].freq as u32;
            },
            0xD410 => {
                self.voices[2].pw_val = (self.voices[2].pw_val & 0x0F00) | value as u16;
            },
            0xD411 => {
                self.voices[2].pw_val = (self.voices[2].pw_val & 0x00FF) | (((value as u16) & 0x000F) << 8);
            },
            0xD412 => {
                self.set_control_register(2, value);
            },
            0xD413 => {
                self.voices[2].attack_add = EG_TABLE[ (value >> 4) as usize ];
                self.voices[2].decay_sub  = EG_TABLE[ (value & 0x0F) as usize ];
            },
            0xD414 => {
                self.voices[2].sustain_level = 0x111111 * (value >> 4) as u32;
                self.voices[2].release_sub   = EG_TABLE[ (value & 0x0F) as usize ];
            },
            0xD416 => {
                if self.filter_freq != value {
                    self.filter_freq = value;
                    self.calculate_filter();
                }
            },
            0xD417 => {
                self.voices[0].filter = (value & 1) != 0;
                self.voices[1].filter = (value & 2) != 0;
                self.voices[2].filter = (value & 4) != 0;
                
                if self.filter_resonance != (value >> 4) {
                    self.filter_resonance = value >> 4;
                    self.calculate_filter();
                }
            },
            0xD418 => {
                self.volume = value & 0x0F;
                self.voices[2].mute = (value & 0x80) != 0;
                let f_type = match (value >> 4) & 7 {
                    0 => FilterType::None,
                    1 => FilterType::Lowpass,
                    2 => FilterType::Bandpass,
                    3 => FilterType::LowBandpass,
                    4 => FilterType::Highpass,
                    5 => FilterType::Notch,
                    6 => FilterType::HighBandpass,
                    7 => FilterType::All,
                    _ => panic!("Impossible filter combination!"),
                };

                if self.filter_type != f_type {
                    self.filter_type = f_type;
                    self.xn1 = 0.0;
                    self.xn2 = 0.0;
                    self.yn1 = 0.0;
                    self.yn2 = 0.0;
                    self.calculate_filter();
                }
            },
            // $D41D-$D41F are unusable, so just ignore it
            0xD420...0xD7FF => self.write_register(0xD400 + (addr % 0x0020), value),
            _               => (),
        }
    }

    // *** private functions *** //

    fn lowpass_resonance(&self, f: f32) -> f32 {
        let f2 = f * f;
        let f3 = f2 * f;
        let f4 = f3 * f;
        227.755 - 1.7635 * f - 0.0176385 * f2 + 0.00333484 * f3 - 9.05683E-6 * f4
    }


    fn highpass_resonance(&self, f: f32) -> f32 {
        let f2 = f * f;
        let f3 = f2 * f;
        366.374 - 14.0052 * f + 0.603212 * f2 - 0.000880196 * f3
    }


    fn calculate_filter(&mut self) {
        let f = self.filter_freq as f32;
        let resonance: f32;
        let mut arg: f32;
        
        match self.filter_type {
            FilterType::None => {
                self.d1 = 0.0;
                self.d2 = 0.0;
                self.g1 = 0.0;
                self.g2 = 0.0;
                self.iir_att = 0.0;
                return;
            },
            FilterType::All => {
                self.d1 = 0.0;
                self.d2 = 0.0;
                self.g1 = 0.0;
                self.g2 = 0.0;
                self.iir_att = 1.0;
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

        self.g2 = 0.55 + 1.2 * arg * arg - 1.2 * arg +  0.0133333333 * self.filter_resonance as f32;
        self.g1 = -2.0 * self.g2.sqrt() * (f32::consts::PI * arg).cos();

        match self.filter_type {
            FilterType::LowBandpass | FilterType::HighBandpass => self.g2 += 0.1,
            _ => ()
        }

        if self.g1.abs() >= (self.g2 + 1.0) {
            if self.g1 > 0.0 { self.g1 = self.g2 + 0.99;    }
            else             { self.g1 = -(self.g2 + 0.99); }
        }

        match self.filter_type {
            FilterType::LowBandpass | FilterType::Lowpass => {
                self.d1 = 2.0;
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


    fn set_control_register(&mut self, v_num: usize, value: u8) {
        self.voices[v_num].wave = match (value >> 4) & 0x0F {
            0 => WaveForm::None,
            1 => WaveForm::Triangle,
            2 => WaveForm::Saw,
            3 => WaveForm::TriSaw,
            4 => WaveForm::Pulse,
            5 => WaveForm::TriPulse,
            6 => WaveForm::SawPulse,
            7 => WaveForm::TriSawPulse,
            8 => WaveForm::Noise,
            _ => panic!("Impossible waveform value!"),
        };
        
        let gate_on = (value & 1) != 0;
        let sync_on = (value & 2) != 0;
        let ring_on = (value & 4) != 0;
        let test_on = (value & 8) != 0;
        
        if gate_on != self.voices[v_num].gate {
            if gate_on {
                self.voices[v_num].state = VoiceState::Attack;
            }
            else {
                match self.voices[v_num].state {
                    VoiceState::Idle => (),
                    _                => self.voices[v_num].state = VoiceState::Release,
                }
            }

            let modulator = self.voices[v_num].modulator;
            self.voices[v_num].gate = gate_on;
            self.voices[modulator].sync = sync_on;
            self.voices[v_num].ring = ring_on;
            self.voices[v_num].test = test_on;

            if test_on {
                self.voices[v_num].wf_cnt = 0;
            }
        } 
    }
}

// SDL2 audio callback implementation - this is where the samples are being converted to output sound
impl AudioCallback for SIDAudioDevice {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        let iir_att = self.iir_att;
        let d1 = self.d1;
        let d2 = self.d2;
        let g1 = self.g1;
        let g2 = self.g2;

        let mut sample_count = (self.sample_idx + NUM_SAMPLES/2) << 16;
        
        for x in out.iter_mut() {
            let master_volume: u8 = self.sample_buffer[(sample_count >> 16) % NUM_SAMPLES];

            sample_count += ((50 * NUM_SAMPLES/2) << 16) / SAMPLE_FREQ as usize;
            let mut total_output: i32 = (SAMPLE_TABLE[master_volume as usize] as i32) << 8;
            let mut total_output_filter: i32 = 0;
            
            for i in 0..3 {
                let envelope: f32;

                match self.voices[i].state
                {
                    VoiceState::Attack => {
                        self.voices[i].level += self.voices[i].attack_add;
                        if self.voices[i].level > 0xFFFFFF {
                            self.voices[i].level = 0xFFFFFF;
                            self.voices[i].state = VoiceState::Decay;
                        }
                    },
                    VoiceState::Decay => {
                        if (self.voices[i].level <= self.voices[i].sustain_level) || (self.voices[i].level > 0xFFFFFF) {
                            self.voices[i].level = self.voices[i].sustain_level;
                        }
                        else {
                            self.voices[i].level -= self.voices[i].decay_sub >> EGDR_SHIFT[ (self.voices[i].level >> 16) as usize ];
                            if (self.voices[i].level <= self.voices[i].sustain_level) || (self.voices[i].level > 0xFFFFFF) {
                                self.voices[i].level = self.voices[i].sustain_level;
                            }
                        }
                    },
                    VoiceState::Release => {
                        self.voices[i].level -= self.voices[i].release_sub >> EGDR_SHIFT[ (self.voices[i].level >> 16) as usize ];
                        if self.voices[i].level > 0xFFFFFF {
                            self.voices[i].level = 0;
                            self.voices[i].state = VoiceState::Idle;
                        }
                    },
                    VoiceState::Idle => {
                        self.voices[i].level = 0;
                    },
                }

                envelope = ((self.voices[i].level as f32) * master_volume as f32) / (0xFFFFFF * 0xF) as f32;
                let modulatee = self.voices[i].modulatee;
                let modulator = self.voices[i].modulator;
                
                if self.voices[i].mute {
                    continue;
                }
                
                if !self.voices[i].test {
                    self.voices[i].wf_cnt += self.voices[i].wf_add;
                }

                if self.voices[i].sync && (self.voices[i].wf_cnt > 0x1000000) {
                    self.voices[modulatee].wf_cnt = 0;
                }

                self.voices[i].wf_cnt &= 0xFFFFFF;

                let mut output: u16 = 0;
                match self.voices[i].wave {
                    WaveForm::Triangle => {
                        unsafe {
                            if self.voices[i].ring {
                                output = TRI_TABLE[((self.voices[i].wf_cnt ^ (self.voices[modulator].wf_cnt & 0x800000)) >> 11) as usize];
                            }
                            else {
                                output = TRI_TABLE[ (self.voices[i].wf_cnt >> 11) as usize ];
                            }
                        }
                    },
                    WaveForm::Saw => {
                        output = (self.voices[i].wf_cnt >> 8) as u16;
                    },
                    WaveForm::Pulse => {
                        if self.voices[i].wf_cnt > (self.voices[i].pw_val << 12) as u32 {
                            output = 0xFFFF;
                        }
                    },
                    WaveForm::TriSaw => {
                        output = TRI_SAW_TABLE[ (self.voices[i].wf_cnt >> 16) as usize ];
                    },
                    WaveForm::TriPulse => {
                        if self.voices[i].wf_cnt > (self.voices[i].pw_val << 12) as u32 {
                            output = TRI_RECT_TABLE[ (self.voices[i].wf_cnt >> 16) as usize ];
                        }
                    },
                    WaveForm::SawPulse => {
                        if self.voices[i].wf_cnt > (self.voices[i].pw_val << 12) as u32 {
                            output = SAW_RECT_TABLE[ (self.voices[i].wf_cnt >> 16) as usize ];
                        }
                    },
                    WaveForm::TriSawPulse => {
                        if self.voices[i].wf_cnt > (self.voices[i].pw_val << 12) as u32 {
                            output = TRI_SAW_RECT_TABLE[ (self.voices[i].wf_cnt >> 16) as usize ];
                        }
                    },
                    WaveForm::Noise => {
                        if self.voices[i].wf_cnt > 0x100000 {
                            let rnd_noise = rand::random::<u16>() << 8;
                            self.voices[i].noise = rnd_noise as u32;
                            output = rnd_noise;
                            self.voices[i].wf_cnt &= 0xFFFFF;
                        }
                        else {
                            output = self.voices[i].noise as u16;
                        }
                    },
                    WaveForm::None => {
                        output = 0x8000;
                    }
                }

                if self.voices[i].filter {
                    total_output_filter += (envelope * ((output >> 3) ^ 0x8000) as f32) as i32;
                }
                else {
                    total_output += (envelope * ((output >> 3) ^ 0x8000) as f32) as i32;
                }
            }

            // take filters into account
            let xn = (total_output_filter * iir_att as i32) as f32;
            let yn = xn + d1 * self.xn1 + d2 * self.xn2 - g1 * self.yn1 - g2 * self.yn2;
            self.yn2 = self.yn1;
            self.yn1 = yn;
            self.xn2 = self.xn1;
            self.xn1 = xn;
            total_output_filter = yn as i32;

            let sample_value = (((total_output + total_output_filter)) >> 2) as i16;

            // output the sample!
            *x = sample_value;
        }
    }
}
