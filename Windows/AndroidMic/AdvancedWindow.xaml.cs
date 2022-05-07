using System;
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
        }

        // update component states
        private void InitComponentStates()
        {
            float val = 0.0f;
            // init pitch shifter states
            PitchShiferEnableCheckbox.IsChecked = audioM.IsEnabled(AdvancedFilterType.FPitchShifter);
            audioM.PipelineFilterConfig(AdvancedFilterType.FPitchShifter, (int)FilterPitchShifter.ConfigTypes.ConfigPitch, ref val, false);
            PitchSlider.Value = val;
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

        //  pitch shifter enable state changed
        private void PitchShiferEnableCheckbox_StateChanged(object sender, RoutedEventArgs e)
        {
            CheckBox checkBox = sender as CheckBox;
            if(checkBox != null)
            {
                audioM?.UpdatePipelineFilter(AdvancedFilterType.FPitchShifter, checkBox.IsChecked == true);
            }
        }
    }
}
