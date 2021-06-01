using System;
using System.Diagnostics;
using System.ComponentModel;
using System.Collections.Generic;
using System.Windows;
using System.Windows.Input;
using System.Windows.Controls;
using System.Windows.Documents;

namespace AndroidMic
{
    // Audio data structure for storing received data
    public class AudioData
    {
        private readonly Queue<Tuple<byte[], int>> buffer = new Queue<Tuple<byte[], int>>();
        private const int MAX_BUFFER_SIZE = 10;
        public void AddData(byte[] data, int realSize)
        {
            lock(this)
            {
                buffer.Enqueue(Tuple.Create(data, realSize));
                while (buffer.Count > MAX_BUFFER_SIZE) buffer.Dequeue();
            }
        }
        public Tuple<byte[], int> GetData()
        {
            lock(this)
            {
                if (buffer.Count > 0) return buffer.Dequeue();
                else return null;
            }
        }
    }

    public partial class MainWindow : Window
    {
        private readonly AudioData mGlobalData = new AudioData();
        private readonly BluetoothHelper mHelperBluetooth;
        private readonly AudioHelper mHelperAudio;
        private readonly USBHelper mHelperUSB;
        public WaveDisplay mWaveformDisplay;
        private readonly System.Windows.Forms.NotifyIcon notifyIcon = new System.Windows.Forms.NotifyIcon();

        public MainWindow()
        {
            InitializeComponent();
            // init objects
            mHelperBluetooth = new BluetoothHelper(this, mGlobalData);
            mHelperAudio = new AudioHelper(this, mGlobalData);
            mHelperUSB = new USBHelper(this, mGlobalData);
            // setup USB ips
            SetupUSBList();
            // setup audio
            SetupAudioList();
            mHelperAudio.Start();
            VolumeSlider.Value = 1.0;
            // set debug information
            Debug.AutoFlush = true;
            // create system tray icon
            // reference: https://stackoverflow.com/questions/10230579/easiest-way-to-have-a-program-minimize-itself-to-the-system-tray-using-net-4
            using (System.IO.Stream iconStream = Application.GetResourceStream(new Uri("/icon.ico", UriKind.Relative)).Stream)
            {
                notifyIcon.Icon = new System.Drawing.Icon(iconStream);
            }
            notifyIcon.Visible = true;
            notifyIcon.DoubleClick +=
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
            // set waveform image
            mWaveformDisplay = new WaveDisplay(WaveformCanvas);
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
            mHelperBluetooth.StopServer();
            mHelperAudio.Stop();
            mHelperUSB.Clean();
            notifyIcon.Dispose();
        }

        // click event for connect button
        private void ConnectButton_Click(object sender, RoutedEventArgs e)
        {
            Button button = (Button)sender;
            if(mHelperBluetooth.Status == BthStatus.DEFAULT)
            {
                if(IsConnected())
                {
                    AddLogMessage("You have already connected");
                }
                else if(!BluetoothHelper.CheckBluetooth())
                {
                    MessageBox.Show("Bluetooth not enabled\nPlease enable it and try again", "AndroidMic Bluetooth", 
                        MessageBoxButton.OK, MessageBoxImage.Information);
                }
                else
                {
                    mHelperBluetooth.StartServer();
                    button.Content = "Disconnect";
                }
            }
            else
            {
                mWaveformDisplay.Reset();
                mHelperBluetooth.StopServer();
                button.Content = "Connect";
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

        // drop down closed for combobox of audio device list
        private void AudioDeviceList_DropDownClosed(object sender, EventArgs e)
        {
            mHelperAudio.SetAudioDevice(AudioDeviceList.SelectedIndex - 1);
            if (mHelperAudio.RefreshAudioDevices()) SetupAudioList();
        }

        // drop down closed for combobox of USB server IP addreses
        private void USBIP_DropDownClosed(object sender, EventArgs e)
        {
            mHelperUSB.StartServer(USBIPAddresses.SelectedIndex - 1);
            if (mHelperUSB.RefreshIpAdress()) SetupUSBList();
        }

        // volume slider change callback
        private void VolumeSlider_PropertyChange(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            mHelperAudio.SetVolume((float)e.NewValue);
        }

        // log message auto scroll to bottom
        private void LogBlockScroll_ScrollChanged(object sender, ScrollChangedEventArgs e)
        {
            if(e.ExtentHeightChange != 0)
            {
                LogBlockScroll.ScrollToVerticalOffset(LogBlockScroll.ExtentHeight);
            }
        }

        // set up audio device list
        private void SetupAudioList()
        {
            AudioDeviceList.Items.Clear();
            AudioDeviceList.Items.Add("Default");
            AudioDeviceList.SelectedIndex = 0;
            string[] devices = mHelperAudio.DeviceList;
            foreach(string device in devices)
            {
                AudioDeviceList.Items.Add(device);
            }
        }

        // set up USB IP list
        private void SetupUSBList()
        {
            USBIPAddresses.Items.Clear();
            USBIPAddresses.Items.Add("Disabled");
            USBIPAddresses.SelectedIndex = 0;
            string[] addresses = mHelperUSB.IPAddresses;
            foreach(string address in addresses)
            {
                USBIPAddresses.Items.Add(address);
            }
        }

        // help function to append log message to text block
        public void AddLogMessage(string message)
        {
            string[] messages = message.Split('\n');
            foreach (string m in messages)
            {
                LogBlock.Inlines.Add(m);
                LogBlock.Inlines.Add(new LineBreak());
            }
        }

        // refresh waveform image data
        public void RefreshWaveform(short valPos, short valNeg)
        {
            // only update when window is not minimized
            if(WindowState != WindowState.Minimized)
                mWaveformDisplay.AddData(valPos, valNeg);
        }

        public bool IsConnected()
        {
            return (mHelperBluetooth.Status == BthStatus.CONNECTED) || (mHelperUSB.Status == USBStatus.CONNECTED);
        }
    }
}
