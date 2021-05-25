using NAudio;
using NAudio.Wave;
using System;
using System.Windows;
using System.Diagnostics;

namespace AndroidMic
{
    class AudioHelper
    {
        private readonly MainWindow mMainWindow;
        private readonly WaveOut mWaveOut;
        public string[] DeviceList { get; private set; }

        public AudioHelper(MainWindow mainWindow)
        {
            mMainWindow = mainWindow;
            mWaveOut = new WaveOut();
            mWaveOut.DeviceNumber = -1; // use default device first
            DeviceList = new string[WaveOut.DeviceCount];
            for (int i = 0; i < DeviceList.Length; i++)
                DeviceList[i] = WaveOut.GetCapabilities(i).ProductName;
        }

        public void SetAudioDevice(int i)
        {
            mWaveOut.DeviceNumber = i;
            AddLog("Device changed to " + ((i < 0) ? "Default" : DeviceList[i]));
        }

        private void AddLog(string message)
        {
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.AddLogMessage("[Audio]\n" + message + "\n");
            }));
        }
    }
}
