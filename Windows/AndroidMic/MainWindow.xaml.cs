using System;
using System.Threading;
using System.Diagnostics;
using System.ComponentModel;
using System.Windows;
using System.Windows.Input;
using System.Windows.Controls;
using System.Windows.Documents;
using AndroidMic.Audio;
using AndroidMic.Streaming;

namespace AndroidMic
{
    public class MessageArgs : EventArgs
    {
        public string Message { get; set; }
    }

    public partial class MainWindow : Window
    {
        private readonly AudioBuffer sharedBuffer;
        private readonly StreamManager streamM;
        private readonly AudioManager audioM;
        private readonly System.Windows.Forms.NotifyIcon notifyIcon;
        private readonly SynchronizationContext uiContext;
        private bool notifDisplayedOnce = false;
        private bool windowInitialized = false;

        [System.Runtime.InteropServices.DllImport("User32.dll")]
        private static extern bool SetForegroundWindow(IntPtr handle);

        public MainWindow()
        {
            // avoid duplicate processes
            Process[] processes = Process.GetProcessesByName(Process.GetCurrentProcess().ProcessName);
            if (processes.Length > 1)
            {
                int prevProcessIdx = 0;
                while (prevProcessIdx < processes.Length &&
                    processes[prevProcessIdx].Id == Process.GetCurrentProcess().Id)
                    prevProcessIdx++;
                if (prevProcessIdx < processes.Length)
                {
                    // bring previous process to foreground
                    SetForegroundWindow(processes[prevProcessIdx].MainWindowHandle);
                }
                // close current process
                Application.Current.Shutdown();
            }
            InitializeComponent();
            // raise process priority to keep connection stable
            Process.GetCurrentProcess().PriorityClass = ProcessPriorityClass.RealTime;
            // set debug information
            Debug.AutoFlush = true;
            // set UI context
            uiContext = SynchronizationContext.Current;
            // init objects
            sharedBuffer = new AudioBuffer();
            streamM = new StreamManager(sharedBuffer);
            streamM.AddLogEvent += Services_AddLogEvent;
            streamM.ServerListeningEvent += StreamM_ServerListeningEvent;
            streamM.ClientConnectedEvent += StreamM_ClientConnectedEvent;
            streamM.ClientDisconnectedEvent += StreamM_ClientDisconnectedEvent;
            audioM = new AudioManager(sharedBuffer);
            audioM.AddLogEvent += Services_AddLogEvent;
            audioM.RefreshAudioDevicesEvent += AudioM_RefreshAudioDevicesEvent;
            audioM.RefreshAudioDevices();
            audioM.ApplyToCanvas(WaveformCanvas);
            // create system tray icon
            notifyIcon = new System.Windows.Forms.NotifyIcon();
            SetupNotificationIcon();
            LoadUserSettings();
            windowInitialized = true;
        }

        private void SetupNotificationIcon()
        {
            // reference: https://stackoverflow.com/questions/10230579/easiest-way-to-have-a-program-minimize-itself-to-the-system-tray-using-net-4
            using (var iconStream = Application.GetResourceStream(new Uri("/icon.ico", UriKind.Relative)).Stream)
            {
                notifyIcon.Icon = new System.Drawing.Icon(iconStream);
            }
            notifyIcon.Visible = true;
            notifyIcon.BalloonTipTitle = Title;
            notifyIcon.BalloonTipText = "App minimized. Click to show.";
            notifyIcon.Click +=
                delegate (object sender, EventArgs e)
                {
                    Show();
                    WindowState = WindowState.Normal;
                };
            notifyIcon.ContextMenu = new System.Windows.Forms.ContextMenu();
            notifyIcon.ContextMenu.MenuItems.Add(
                "Quit",
                delegate (object sender, EventArgs e)
                {
                    Close();
                }
            );
        }

        // load user settings for MainWindow & AdvancedWindow
        private void LoadUserSettings()
        {
            var settings = Properties.Settings.Default;
            // MainWindow_AudioDisplayEnabled
            if (!settings.MainWindow_AudioDisplayEnabled)
            {
                var margin = LogBlockBorder.Margin;
                margin.Bottom = 40;
                LogBlockBorder.Margin = margin;
                WaveformCanvasBorder.Height = 20;
            }
            // MainWindow_AudioDeviceIdx
            {
                int idx = settings.MainWindow_AudioDeviceIdx;
                audioM.SelectAudioDevice(idx);
            }
            // MainWindow_AudioVolume
            {
                float volume = settings.MainWindow_AudioVolume;
                audioM.SetVolume(volume);
                VolumeSlider.Value = volume;
            }
            // MainWindow_ConnectViaBluetooth
            if (settings.MainWindow_ConnectViaBluetooth)
            {
                streamM.SetConnectionType(StreamManager.ConnectionType.BLUETOOTH);
                RadioButton1.IsChecked = true;
            }
            else
            {
                streamM.SetConnectionType(StreamManager.ConnectionType.WIFI);
                RadioButton2.IsChecked = true;
            }

            float val;
            // AdvancedWindow_PitchShifterEnabled
            {
                audioM.UpdatePipelineFilter(AdvancedFilterType.FPitchShifter, settings.AdvancedWindow_PitchShifterEnabled);
            }
            // AdvancedWindow_PitchShifterVal
            {
                val = settings.AdvancedWindow_PitchShifterVal;
                audioM.PipelineFilterConfig(AdvancedFilterType.FPitchShifter, (int)FilterPitchShifter.ConfigTypes.ConfigPitch, ref val, true);
            }
            // AdvancedWindow_WhiteNoiseEnabled
            {
                audioM.UpdatePipelineFilter(AdvancedFilterType.FWhiteNoise, settings.AdvancedWindow_WhiteNoiseEnabled);
            }
            // AdvancedWindow_WhiteNoiseVal
            {
                val = settings.AdvancedWindow_WhiteNoiseVal;
                audioM.PipelineFilterConfig(AdvancedFilterType.FWhiteNoise, (int)FilterWhiteNoise.ConfigTypes.ConfigStrength, ref val, true);
            }
            // AdvancedWindow_RepeatTrackEnabled
            {
                audioM.UpdatePipelineFilter(AdvancedFilterType.FRepeatTrack, settings.AdvancedWindow_RepeatTrackEnabled);
            }
            // AdvancedWindow_RepeatTrackStrength
            {
                val = settings.AdvancedWindow_RepeatTrackStrength;
                audioM.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigStrength, ref val, true);
            }
            // AdvancedWindow_RepeatTrackLoop
            {
                val = settings.AdvancedWindow_RepeatTrackLoop;
                audioM.PipelineFilterConfig(AdvancedFilterType.FRepeatTrack, (int)FilterRepeatTrack.ConfigTypes.ConfigRepeat, ref val, true);
            }

            bool valB;
            // AdvancedWindow_SpeexDenoise
            {
                valB = settings.AdvancedWindow_SpeexDenoise;
                audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigDenoise, ref valB, true);
            }
            // AdvancedWindow_SpeexAGC
            {
                valB = settings.AdvancedWindow_SpeexAGC;
                audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigAGC, ref valB, true);
            }
            // AdvancedWindow_SpeexVAD
            {
                valB = settings.AdvancedWindow_SpeexVAD;
                audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigVAD, ref valB, true);
            }
            // AdvancedWindow_SpeexEcho
            {
                valB = settings.AdvancedWindow_SpeexEcho;
                audioM.ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes.ConfigEcho, ref valB, true);
            }
        }

        // check window state change
        protected override void OnStateChanged(EventArgs e)
        {
            if (WindowState == WindowState.Minimized)
            {
                if (!notifDisplayedOnce)
                {
                    notifyIcon.ShowBalloonTip(2000, "AndroidMic", "App minimized to system tray", System.Windows.Forms.ToolTipIcon.Info);
                    notifDisplayedOnce = true;
                }
                Hide();
            }
            base.OnStateChanged(e);
        }

        // close event for main window
        private void MainWindow_Closing(object sender, CancelEventArgs e)
        {
            streamM?.Stop();
            audioM?.Shutdown();
            notifyIcon?.Dispose();
            Properties.Settings.Default.Save();
        }

        // click event for connect button
        private void ConnectButton_Click(object sender, RoutedEventArgs e)
        {
            Button btn = sender as Button;
            if (btn != null)
            {
                if (btn.Content.ToString().StartsWith("C"))
                    streamM?.Start();
                else
                    streamM?.Stop();
            }
        }

        // click event for advanced button
        private void AdvancedButton_Click(object sender, RoutedEventArgs e)
        {
            AdvancedWindow advancedWindow = new AdvancedWindow(audioM)
            {
                Owner = this
            };
            advancedWindow.Show();
        }

        // mouse down and check double click for log message block
        private void LogBlock_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ClickCount == 2)
            {
                LogBlock.Inlines.Clear(); // clear message if double clicked
            }
        }

        // log message auto scroll to bottom
        private void LogBlockScroll_ScrollChanged(object sender, ScrollChangedEventArgs e)
        {
            if (e.ExtentHeightChange != 0)
            {
                LogBlockScroll.ScrollToVerticalOffset(LogBlockScroll.ExtentHeight);
            }
        }

        // mouse down and check double click for audio visualization
        private void AudioDisplay_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ClickCount == 2)
            {
                if (sender is Border b)
                {
                    bool enabled = audioM.ToggleCanvas();
                    if (enabled)
                    {
                        var margin = LogBlockBorder.Margin;
                        margin.Bottom = 140;
                        LogBlockBorder.Margin = margin;
                        b.Height = 100;
                        audioM.ApplyToCanvas(WaveformCanvas);
                    }
                    else
                    {
                        var margin = LogBlockBorder.Margin;
                        margin.Bottom = 40;
                        LogBlockBorder.Margin = margin;
                        b.Height = 20;
                        WaveformCanvas.Children.Clear();
                    }
                    Properties.Settings.Default.MainWindow_AudioDisplayEnabled = enabled;
                }
            }
        }

        // drop down closed for combobox of audio device list
        private void AudioDeviceList_DropDownClosed(object sender, EventArgs e)
        {
            int idx = AudioDeviceList.SelectedIndex - 1;
            audioM?.SelectAudioDevice(idx);
            Properties.Settings.Default.MainWindow_AudioDeviceIdx = idx;
        }

        // connection type radio button checked event
        private void RadioButton_Checked(object sender, RoutedEventArgs e)
        {
            RadioButton rb = sender as RadioButton;
            if (rb != null && (rb.IsChecked == true))
            {
                // select bluetooth or wifi
                if (rb.Content.ToString().StartsWith("B"))
                {
                    streamM?.SetConnectionType(StreamManager.ConnectionType.BLUETOOTH);
                    if (windowInitialized)
                        Properties.Settings.Default.MainWindow_ConnectViaBluetooth = true;
                }
                else
                {
                    streamM?.SetConnectionType(StreamManager.ConnectionType.WIFI);
                    if (windowInitialized)
                        Properties.Settings.Default.MainWindow_ConnectViaBluetooth = false;
                }
            }
        }

        // volume slider change callback
        private void VolumeSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            float volume = (float)e.NewValue;
            audioM?.SetVolume(volume);
            if (windowInitialized)
                Properties.Settings.Default.MainWindow_AudioVolume = volume;
        }

        // add log message to UI
        private void Services_AddLogEvent(object sender, MessageArgs e)
        {
            string[] messages = e.Message.Split('\n');
            uiContext?.Post(delegate
            {
                lock (LogBlock)
                {
                    foreach (string m in messages)
                    {
                        LogBlock.Inlines.Add(m);
                        LogBlock.Inlines.Add(new LineBreak());
                    }
                }
            }, null);
        }

        // refresh audio devices list
        private void AudioM_RefreshAudioDevicesEvent(object sender, AudioDevicesArgs e)
        {
            uiContext?.Post(delegate
            {
                lock (AudioDeviceList)
                {
                    AudioDeviceList.Items.Clear();
                    AudioDeviceList.Items.Add("Default");
                    foreach (string device in e.Devices)
                    {
                        AudioDeviceList.Items.Add(device);
                    }
                    AudioDeviceList.SelectedIndex = Math.Min(e.SelectedIdx + 1, AudioDeviceList.Items.Count - 1);
                }
            }, null);
        }

        // server starts listening
        private void StreamM_ServerListeningEvent(object sender, EventArgs e)
        {
            uiContext?.Post(delegate
            {
                lock (RadioButton1)
                {
                    RadioButton1.IsEnabled = false;
                    RadioButton2.IsEnabled = false;
                }
                lock (ConnectButton)
                {
                    ConnectButton.Content = "Listening";
                    ConnectButton.ToolTip = "Click to Stop";
                }
            }, null);
        }

        // client connected
        private void StreamM_ClientConnectedEvent(object sender, EventArgs e)
        {
            uiContext?.Post(delegate
            {
                lock (RadioButton1)
                {
                    RadioButton1.IsEnabled = false;
                    RadioButton2.IsEnabled = false;
                }
                lock (ConnectButton)
                {
                    ConnectButton.Content = "Disconnect";
                    ConnectButton.ToolTip = "Click to Disconnect";
                }
            }, null);
        }

        // client disconnected
        private void StreamM_ClientDisconnectedEvent(object sender, EventArgs e)
        {
            uiContext?.Post(delegate
            {
                lock (RadioButton1)
                {
                    RadioButton1.IsEnabled = true;
                    RadioButton2.IsEnabled = true;
                }
                lock (ConnectButton)
                {
                    ConnectButton.Content = "Connect";
                    ConnectButton.ToolTip = "Start Server";
                }
            }, null);
        }
    }
}
