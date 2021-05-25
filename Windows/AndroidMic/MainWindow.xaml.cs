using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Documents;

namespace AndroidMic
{
    public partial class MainWindow : Window
    {
        private BluetoothHelper mHelperBluetooth;
        private AudioHelper mHelperAudio;

        public MainWindow()
        {
            InitializeComponent();
            mHelperBluetooth = new BluetoothHelper(this);
            mHelperAudio = new AudioHelper(this);
            SetupAudioList();
        }

        private void MainWindow_Closing(object sender, CancelEventArgs e)
        {
            mHelperBluetooth.StopServer();
        }

        private void ConnectButton_Click(object sender, RoutedEventArgs e)
        {
            Button button = (Button)sender;
            if(mHelperBluetooth.Status == BthStatus.DEFAULT)
            {
                if(!BluetoothHelper.CheckBluetooth())
                {
                    MessageBox.Show("Bluetooth not enabled\nPlease enable it and try again", "AndroidMic Bluetooth", MessageBoxButton.OK);
                }
                else
                {
                    mHelperBluetooth.StartServer();
                    button.Content = "Disconnect";
                }
            }
            else
            {
                mHelperBluetooth.StopServer();
                button.Content = "Connect";
            }
        }

        private void AudioDeviceList_DropDownClosed(object sender, EventArgs e)
        {
            mHelperAudio.SetAudioDevice(AudioDeviceList.SelectedIndex - 1);
        }

        private void SetupAudioList()
        {
            string[] devices = mHelperAudio.DeviceList;
            foreach(string device in devices)
            {
                AudioDeviceList.Items.Add(device);
            }
        }

        public void AddLogMessage(string message)
        {
            string[] messages = message.Split('\n');
            foreach (string m in messages)
            {
                LogBlock.Inlines.Add(m);
                LogBlock.Inlines.Add(new LineBreak());
            }
        }
    }
}
