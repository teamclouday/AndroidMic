using System.Windows;
using System.Windows.Controls;
using AndroidMic.Audio;

namespace AndroidMic
{
    /// <summary>
    /// Interaction logic for AdvancedWindow.xaml
    /// </summary>
    public partial class AdvancedWindow : Window
    {
        private readonly AudioManager audioM;
        private bool windowInitialized = false;

        public AdvancedWindow(AudioManager audioM)
        {
            InitializeComponent();
            this.audioM = audioM;
            InitComponentStates();
            // prevent parent hiding on close
            Closing += (a, b) => { Application.Current.MainWindow.Activate(); };
            windowInitialized = true;
        }

        // update component states
        private void InitComponentStates()
        {
            float val = 0.0f;
            // init pitch shifter states
            PitchShiferEnableCheckbox.IsChecked = audioM.IsEnabled(AdvancedFilterType.FPitchShifter);
            audioM.PipelineFilterConfig(AdvancedFilterType.FPitchShifter, (int)FilterPitchShifter.ConfigTypes.ConfigPitch, ref val, false);
            PitchSlider.Value = val;
            // init white noise states
            WhiteNoiseEnableCheckbox.IsChecked = audioM.IsEnabled(AdvancedFilterType.FWhiteNoise);
            audioM.PipelineFilterConfig(AdvancedFilterType.FWhiteNoise, (int)FilterWhiteNoise.ConfigTypes.ConfigStrength, ref val, false);
            NoiseRatioSlider.Value = val;
            // init repeat track states
            RepeatTrackEnableCheckbox.IsChecked = audioM.IsEnabled(AdvancedFilterType.FRepeatTrack);
            audioM.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigStrength, ref val, false);
            TrackRatioSlider.Value = val;
            audioM.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigRepeat, ref val, false);
            RepeatTrackLoopCheckbox.IsChecked = val == 1.0f;
            // desired latency
            LatencySlider.Value = audioM.PlayerDesiredLatency;
            // init speex states
            audioM.SetIndicator(SpeechIndicator);
            bool valB = false;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigDenoise, ref valB, false);
            NoiseCancelEnableCheckbox.IsChecked = valB;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigAGC, ref valB, false);
            AutomicGainEnableCheckbox.IsChecked = valB;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigVAD, ref valB, false);
            VADEnableCheckbox.IsChecked = valB;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigEcho, ref valB, false);
            EchoCancelEnableCheckbox.IsChecked = valB;
        }

        // pitch slider change callback
        private void PitchSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            Slider slider = sender as Slider;
            if (slider != null)
            {
                float val = (float)slider.Value;
                audioM?.PipelineFilterConfig(AdvancedFilterType.FPitchShifter, (int)FilterPitchShifter.ConfigTypes.ConfigPitch, ref val, true);
                Properties.Settings.Default.AdvancedWindow_PitchShifterVal = val;
            }
        }

        // pitch shifter enable state changed
        private void PitchShiferEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool isChecked = checkBox.IsChecked == true;
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FPitchShifter, isChecked);
                Properties.Settings.Default.AdvancedWindow_PitchShifterEnabled = isChecked;
            }
        }

        // white noise slider change callback
        private void NoiseRatioSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            Slider slider = sender as Slider;
            if (slider != null)
            {
                float val = (float)slider.Value;
                audioM?.PipelineFilterConfig(AdvancedFilterType.FWhiteNoise, (int)FilterWhiteNoise.ConfigTypes.ConfigStrength, ref val, true);
                Properties.Settings.Default.AdvancedWindow_WhiteNoiseVal = val;
            }
        }

        // white noise enable state changed
        private void WhiteNoiseEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool isChecked = checkBox.IsChecked == true;
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FWhiteNoise, isChecked);
                Properties.Settings.Default.AdvancedWindow_WhiteNoiseEnabled = isChecked;
            }
        }

        // repeat track slider change callback
        private void TrackRatioSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            Slider slider = sender as Slider;
            if (slider != null)
            {
                float val = (float)slider.Value;
                audioM?.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigStrength, ref val, true);
                Properties.Settings.Default.AdvancedWindow_RepeatTrackStrength = val;
            }
        }

        // repeat track enable state changed
        private void RepeatTrackEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool isChecked = checkBox.IsChecked == true;
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FRepeatTrack, isChecked);
                Properties.Settings.Default.AdvancedWindow_RepeatTrackEnabled = isChecked;
            }
        }

        // track repeat state changed
        private void RepeatTrackLoopCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                float val = checkBox.IsChecked == true ? 1.0f : 0.0f;
                audioM?.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigRepeat, ref val, true);
                Properties.Settings.Default.AdvancedWindow_RepeatTrackLoop = val;
            }
        }

        // select file button click event
        private void SelectFileButton_Click(object sender, RoutedEventArgs e)
        {
            Button button = sender as Button;
            if (button != null)
            {
                float val = 0.0f;
                audioM?.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigSelectFile, ref val, true);
            }
        }

        // expanded expander
        private void Expander_Expanded(object sender, RoutedEventArgs e)
        {
            Expander expander = sender as Expander;
            if (expander != null)
            {
                CloseAllExpanders(expander);
                Panel.SetZIndex(expander, 5);
            }
        }

        // expander closed
        private void Expander_Collapsed(object sender, RoutedEventArgs e)
        {
            Expander expander = sender as Expander;
            if (expander != null)
            {
                Panel.SetZIndex(expander, 0);
            }
        }

        // close all expanders
        private void CloseAllExpanders(Expander exception)
        {
            if (!exception.Equals(Expander1))
                Expander1.IsExpanded = false;
            if (!exception.Equals(Expander2))
                Expander2.IsExpanded = false;
            if (!exception.Equals(Expander3))
                Expander3.IsExpanded = false;
        }

        // volume slider change callback
        private void LatencySlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            int desiredLatency = (int)e.NewValue;
            if (windowInitialized)
            {
                audioM.PlayerDesiredLatency = desiredLatency;
                Properties.Settings.Default.MainWindow_PlayerDesiredLatency = desiredLatency;
            }
        }

        // noise cancelling enable state changed
        private void NoiseCancelEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool enabled = checkBox.IsChecked == true;
                audioM?.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigDenoise, ref enabled, true);
                Properties.Settings.Default.AdvancedWindow_SpeexDenoise = enabled;
            }
        }

        // AGC enable state changed
        private void AutomicGainEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool enabled = checkBox.IsChecked == true;
                audioM?.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigAGC, ref enabled, true);
                Properties.Settings.Default.AdvancedWindow_SpeexAGC = enabled;
            }
        }

        // VAD enable state changed
        private void VADEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool enabled = checkBox.IsChecked == true;
                audioM?.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigVAD, ref enabled, true);
                Properties.Settings.Default.AdvancedWindow_SpeexVAD = enabled;
            }
        }

        // echo cancellation enable state changed
        private void EchoCancelEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool enabled = checkBox.IsChecked == true;
                audioM?.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigEcho, ref enabled, true);
                Properties.Settings.Default.AdvancedWindow_SpeexEcho = enabled;
            }
        }
    }
}
