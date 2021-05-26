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
        private Queue<Tuple<byte[], int>> buffer = new Queue<Tuple<byte[], int>>();
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
        private BluetoothHelper mHelperBluetooth;
        private AudioHelper mHelperAudio;
        private AudioData mGlobalData = new AudioData();

        public MainWindow()
        {
            InitializeComponent();
            mHelperBluetooth = new BluetoothHelper(this, mGlobalData);
            mHelperAudio = new AudioHelper(this, mGlobalData);
            SetupAudioList();
            mHelperAudio.Start();
            Debug.AutoFlush = true;
        }

        // close event for main window
        private void MainWindow_Closing(object sender, CancelEventArgs e)
        {
            mHelperBluetooth.StopServer();
            mHelperAudio.Stop();
        }

        // click event for connect button
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
    }
}
