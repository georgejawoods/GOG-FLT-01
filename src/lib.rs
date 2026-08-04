use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

mod ui;

#[derive(Enum, PartialEq, Clone, Copy)]
pub enum FilterType {
    #[name = "Lowpass"]
    Lowpass,
    #[name = "Highpass"]
    Highpass,
    #[name = "Bandpass"]
    Bandpass,
    #[name = "Notch"]
    Notch,
}

// Структура плагина
pub struct Flt01 {
    params: Arc<Flt01Params>,
    sample_rate: f32,

    bp_l: f32,
    lp_l: f32,

    bp_r: f32,
    lp_r: f32,
}

// Параметры
#[derive(Params)]
struct Flt01Params {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "filter_type"]
    pub filter_type: EnumParam<FilterType>,

    #[id = "cutoff"]
    pub cutoff: FloatParam,

    #[id = "resonance"]
    pub resonance: FloatParam,

    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "slope"]
    pub slope: BoolParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "out_level"]
    pub out_level: FloatParam,
}

impl Default for Flt01 {
    fn default() -> Self {
        Self { 
            params: Arc::new(Flt01Params {
                editor_state: EguiState::from_size(340, 740),

                filter_type: EnumParam::new(
                    "Type",
                    FilterType::Lowpass,
                ),

            cutoff: FloatParam::new(
                "Cutoff",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_step_size(0.1)
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            resonance: FloatParam::new(
                "Resonance", 
                0.0, 
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_unit(" %")
            .with_step_size(1.0)
            .with_smoother(SmoothingStyle::Linear(50.0)),
            
            drive: FloatParam::new(
                "Drive",
                1.0,
                FloatRange::Linear { min: 1.0, max: 20.0 },
            )
            .with_step_size(0.1)
            .with_smoother(SmoothingStyle::Linear(50.0)),

            slope: BoolParam::new("Slope 24dB", false),

            mix: FloatParam::new(
                "MIX",
                100.0,
                FloatRange::Linear { min: 0.0, max: 100.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %")
            .with_step_size(0.1),

            out_level: FloatParam::new(
                "OUT LEVEL", 
                0.0, 
                FloatRange::Linear { min: -24.0, max: 12.0 },
            )
            .with_unit(" dB")
            .with_step_size(0.01)
            .with_smoother(SmoothingStyle::Linear(50.0)),
            }),

            sample_rate: 44100.0,

            bp_l: 0.0,
            lp_l: 0.0,
            bp_r: 0.0,
            lp_r: 0.0,
        }
    }
}

impl Plugin for Flt01 {
    const NAME: &'static str = "GOG // FLT-01";
    const VENDOR: &'static str = "GOGIK";
    const URL: &'static str = "https://gogik-audio.github.io";
    const EMAIL: &'static str = "gogik.audio@gmail.com";
    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            // Получаем сглаженные значения текущего семпла
            let cutoff = self.params.cutoff.smoothed.next();
            let raw_resonance = self.params.resonance.smoothed.next();
            let filter_type = self.params.filter_type.value();
            let drive = self.params.drive.smoothed.next();

            let mix_val = self.params.mix.smoothed.next() / 100.0;

            let out_gain_db = self.params.out_level.smoothed.next();
            let out_gain_linear = nih_plug::util::db_to_gain(out_gain_db);

            // Ограничиваем срез
            let clamped_cutoff = cutoff.clamp(20.0, self.sample_rate / 2.1);
            let g = 2.0 * (std::f32::consts::PI * clamped_cutoff / self.sample_rate).tan();
            let res_normalized = raw_resonance / 100.0;
            // Коэффициент демпфирования
            let k = 2.0 - (1.9 * res_normalized);
            let a1 = 1.0 / (1.0 + g * k + g *g);
            let a2 = g * a1;
            let a3 = g * a2;

            // Обработка левого канала
            if let Some(left) = channel_samples.get_mut(0) {
                let clean_left = *left;
                let distorted_left = (clean_left * drive).tanh();

                let v3 = distorted_left - self.lp_l;
                let v1 = a1 *self.bp_l + a2 * v3;
                let v2 = self.lp_l + a2 * self.bp_l + a3 * v3;

                self.bp_l = 2.0 * v1 - self.bp_l;
                self.lp_l = 2.0 * v2 - self.lp_l;

                let hp = distorted_left - k * v1 - v2;

                let wet_left = match filter_type {
                    FilterType::Lowpass => self.lp_l,
                    FilterType::Highpass => hp,
                    FilterType::Bandpass => self.bp_l,
                    FilterType::Notch => hp + self.lp_l,
                };

                let mixed_left = clean_left * (1.0 - mix_val) + wet_left * mix_val;
                *left = mixed_left * out_gain_linear;
            }

            // Обработка правого канала
            if let Some(right) = channel_samples.get_mut(1) {
                let clean_right = *right;
                let distorted_right = (*right * drive).tanh();
                
                let v3 = distorted_right - self.lp_r;
                let v1 = a1 * self.bp_r + a2 * v3;
                let v2 = self.lp_r + a2 * self.bp_r + a3 * v3;

                self.bp_r = 2.0 * v1 - self.bp_r;
                self.lp_r = 2.0 * v2 - self.lp_r;

                let hp = distorted_right - k * v1 - v2;

                let wet_right = match filter_type {
                    FilterType::Lowpass => self.lp_r,
                    FilterType::Highpass => hp,
                    FilterType::Bandpass => self.bp_r,
                    FilterType::Notch => hp + self.lp_r,
                };

                let mixed_right = clean_right * (1.0 - mix_val) + wet_right * mix_val;
                *right = mixed_right * out_gain_linear;
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        ui::create_ui(
            self.params.editor_state.clone(),
            self.params.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool
    {
        self.sample_rate = buffer_config.sample_rate;

        self.bp_l = 0.0;
        self.lp_l = 0.0;
        self.bp_r = 0.0;
        self.lp_r = 0.0;

        true
    }
}

impl Vst3Plugin for Flt01 {
    const VST3_CLASS_ID: [u8; 16] = *b"GogikFlt01Plg001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Tools,
    ];
}

impl ClapPlugin for Flt01 {
    const CLAP_ID: &'static str = "audio.gogik.flt-01";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("GOGIK FLT-01 Module");
    const CLAP_MANUAL_URL: Option<&'static str> = Some("https://gogik-audio.github.io");
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
    ];
}

nih_export_vst3!(Flt01);
nih_export_clap!(Flt01);