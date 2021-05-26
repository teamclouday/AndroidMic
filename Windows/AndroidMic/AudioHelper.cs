using NAudio.Wave;
using System;
using System.Windows;
using System.Threading;
using System.Diagnostics;

namespace AndroidMic
{
    class AudioHelper
    {
        private readonly MainWindow mMainWindow;
        private readonly AudioData mGlobalData;
        private readonly WaveFormat mWaveFormat = new WaveFormat(44100, 16, 1); // sample rate, bits, channels
        private readonly int MAX_WAIT_TIME = 1500;
        private readonly int BUFFER_SIZE = 2000;

        public string[] DeviceList { get; private set; }
        private bool isAudioAllowed = false;
        private Thread mProcessThread = null;
        private WaveOut mWaveOut;

        private BufferedWaveProvider mProvider;

        public AudioHelper(MainWindow mainWindow, AudioData globalData)
        {
            mMainWindow = mainWindow;
            mGlobalData = globalData;
            mWaveOut = new WaveOut();
            mWaveOut.DeviceNumber = -1; // use default device first
            RefreshAudioDevices();
        }

        // start playing audio
        public void Start()
        {
            if (mWaveOut.PlaybackState == PlaybackState.Playing) mWaveOut.Stop();
            mProvider = new BufferedWaveProvider(mWaveFormat);
            mProvider.BufferLength = BUFFER_SIZE * 16;
            mProvider.DiscardOnBufferOverflow = true;
            mWaveOut.Init(mProvider);
            mWaveOut.Play();
            isAudioAllowed = true;
            mProcessThread = new Thread(new ThreadStart(Process));
            mProcessThread.Start();
            Debug.WriteLine("[AudioHelper] process thread started");
        }

        // stop playing audio
        public void Stop()
        {
            isAudioAllowed = false;
            if (mProcessThread != null && mProcessThread.IsAlive)
            {
                try
                {
                    if (!mProcessThread.Join(MAX_WAIT_TIME))
                        mProcessThread.Abort();
                }
                catch (ThreadStateException) { }
            }
            if (mWaveOut.PlaybackState == PlaybackState.Playing) mWaveOut.Stop();
            mWaveOut.Dispose();
            Debug.WriteLine("[AudioHelper] stopped");
        }

        // retrieve audio data and add to samples
        private void Process()
        {
            while(isAudioAllowed)
            {
                Tuple<byte[], int> data = mGlobalData.GetData();
                if (data == null)
                {
                    Thread.Sleep(5); // wait for data
                    continue;
                }
                mProvider.AddSamples(data.Item1, 0, data.Item2);
                //if(mWaveOut.PlaybackState != PlaybackState.Playing) mWaveOut.Play();
                Debug.WriteLine("[AudioHelper] new data (" + data.Item2 + " bytes)");
                Thread.Sleep(1);
            }
        }

        // set new audio device
        public void SetAudioDevice(int i)
        {
            if (mWaveOut.PlaybackState == PlaybackState.Playing) mWaveOut.Stop();
            mWaveOut.Dispose();
            mWaveOut = new WaveOut();
            mWaveOut.DeviceNumber = i;
            mWaveOut.Init(mProvider);
            mWaveOut.Play();
            Debug.WriteLine("[AudioHelper] device index changed to " + i);
            AddLog("Device changed to " + ((i < 0) ? "Default" : DeviceList[i]));
        }

        // helper function to add log message
        private void AddLog(string message)
        {
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.AddLogMessage("[Audio]\n" + message + "\n");
            }));
        }

        // refresh audio devices
        public bool RefreshAudioDevices()
        {
            bool changed = false;
            if (DeviceList == null || WaveOut.DeviceCount != DeviceList.Length)
            {
                DeviceList = new string[WaveOut.DeviceCount];
                changed = true;
            }
            for (int i = 0; i < DeviceList.Length; i++)
            {
                string name = WaveOut.GetCapabilities(i).ProductName;
                if (DeviceList[i] != name)
                {
                    changed = true;
                    DeviceList[i] = name;
                }
            }

            return changed;
        }
    }
}
