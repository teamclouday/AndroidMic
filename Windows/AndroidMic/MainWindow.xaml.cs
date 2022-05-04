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

        public MainWindow()
        {
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
            audioM.RendererProvider.RefreshCanvasEvent += AudioM_RefreshCanvasEvent;
            // create system tray icon
            notifyIcon = new System.Windows.Forms.NotifyIcon();
            SetupNotificationIcon();
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

        // check window state change
        protected override void OnStateChanged(EventArgs e)
        {
            if (WindowState == WindowState.Minimized)
            {
                notifyIcon.ShowBalloonTip(2000, "AndroidMic", "App minimized to system tray", System.Windows.Forms.ToolTipIcon.Info);
                Hide();
            }
            base.OnStateChanged(e);
        }

        // close event for main window
        private void MainWindow_Closing(object sender, CancelEventArgs e)
        {
            streamM?.Shutdown();
            audioM?.Shutdown();
            notifyIcon?.Dispose();
        }

        // click event for connect button
        private void ConnectButton_Click(object sender, RoutedEventArgs e)
        {
            Button btn = sender as Button;
            if(btn != null)
            {
                if(btn.Content.ToString().StartsWith("C"))
                    streamM?.Start();
                else
                    streamM?.Stop();
            }
        }

        // mouse down and check double click for log message block
        private void LogBlock_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if(e.ClickCount == 2)
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

        // drop down closed for combobox of audio device list
        private void AudioDeviceList_DropDownClosed(object sender, EventArgs e)
        {
            audioM?.SelectAudioDevice(AudioDeviceList.SelectedIndex - 1);
        }

        // connection type radio button checked event
        private void RadioButton_Checked(object sender, RoutedEventArgs e)
        {
            RadioButton rb = sender as RadioButton;
            if(rb != null && (rb.IsChecked == true))
            {
                // select bluetooth or wifi
                if (rb.Content.ToString().StartsWith("B"))
                    streamM?.SetConnectionType(StreamManager.ConnectionType.BLUETOOTH);
                else
                    streamM?.SetConnectionType(StreamManager.ConnectionType.WIFI);
            }
        }

        // volume slider change callback
        private void VolumeSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            audioM?.SetVolume((float)e.NewValue);
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
                lock(AudioDeviceList)
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
                lock(RadioButton1)
                {
                    RadioButton1.IsEnabled = false;
                    RadioButton2.IsEnabled = false;
                }
                lock(ConnectButton)
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

        private void AudioM_RefreshCanvasEvent(object sender, CanvasEventArgs e)
        {
            uiContext?.Post(delegate
            {
                lock (WaveformCanvas)
                {
                    WaveformCanvas.Children.Clear();
                    WaveformCanvas.Children.Add(e.Data);
                }
            }, null);
        }
    }
}
