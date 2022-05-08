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

        public AdvancedWindow(AudioManager audioM)
        {
            InitializeComponent();
            this.audioM = audioM;
            InitComponentStates();
            // prevent parent hiding on close
            Closing += (a, b) => { Application.Current.MainWindow.Activate(); };
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
            // init speex states
            audioM.SetIndicator(SpeechIndicator);
            bool valB = false;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigDenoise, ref valB, false);
            NoiseCancelEnableCheckbox.IsChecked = valB;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigAGC, ref valB, false);
            AutomicGainEnableCheckbox.IsChecked = valB;
            audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigVAD, ref valB, false);
            VADEnableCheckbox.IsChecked = valB;
        }

        // pitch slider change callback
        private void PitchSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            Slider slider = sender as Slider;
            if(slider != null)
            {
                float val = (float)slider.Value;
                audioM?.PipelineFilterConfig(AdvancedFilterType.FPitchShifter, (int)FilterPitchShifter.ConfigTypes.ConfigPitch, ref val, true);
            }
        }

        // pitch shifter enable state changed
        private void PitchShiferEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if(checkBox != null)
            {
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FPitchShifter, checkBox.IsChecked == true);
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
            }
        }

        // white noise enable state changed
        private void WhiteNoiseEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FWhiteNoise, checkBox.IsChecked == true);
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
            }
        }

        // repeat track enable state changed
        private void RepeatTrackEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FRepeatTrack, checkBox.IsChecked == true);
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
            }
        }

        // select file button click event
        private void SelectFileButton_Click(object sender, RoutedEventArgs e)
        {
            Button button = sender as Button;
            if(button != null)
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
            if(expander != null)
            {
                Panel.SetZIndex(expander, 0);
            }
        }

        // close all expanders
        private void CloseAllExpanders(Expander exception)
        {
            if(!exception.Equals(Expander1))
                Expander1.IsExpanded = false;
            if (!exception.Equals(Expander2))
                Expander2.IsExpanded = false;
            if (!exception.Equals(Expander3))
                Expander3.IsExpanded = false;
        }

        // noise cancelling enable state changed
        private void NoiseCancelEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if (checkBox != null)
            {
                bool enabled = checkBox.IsChecked == true;
                audioM?.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigDenoise, ref enabled, true);
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
            }
        }
    }
}
