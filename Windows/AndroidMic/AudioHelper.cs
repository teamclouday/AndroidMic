using NAudio.Wave;
using NAudio.Wave.SampleProviders;
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

        public string[] DeviceList { get; private set; }
        private bool isAudioAllowed = false;
        private Thread mProcessThread = null;
        private WaveOut mWaveOut;

        private BufferedWaveProvider mBufferedProvider;
        private VolumeSampleProvider mVolumeProvider;

        // render waveform
        private readonly int RENDER_SCREEN_SIZE = 2048; // screen size to update waveform
        private int mRenderByteCount = 0;
        private bool mRenderSkipByte = false; // whether to skip first byte for short alignment
        private short mRenderPos = 0, mRenderNeg = 0;


        public AudioHelper(MainWindow mainWindow, AudioData globalData)
        {
            mMainWindow = mainWindow;
            mGlobalData = globalData;
            mWaveOut = new WaveOut
            {
                DeviceNumber = -1 // use default device first
            };
            RefreshAudioDevices();
        }

        // start playing audio
        public void Start()
        {
            if (mWaveOut.PlaybackState == PlaybackState.Playing) mWaveOut.Stop();
            mBufferedProvider = new BufferedWaveProvider(mWaveFormat)
            {
                DiscardOnBufferOverflow = true
            };
            mVolumeProvider = new VolumeSampleProvider(mBufferedProvider.ToSampleProvider())
            {
                Volume = 1.0f
            };
            mWaveOut.Init(mVolumeProvider);
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
            byte[] buffer = new byte[2];
            while (isAudioAllowed)
            {
                Tuple<byte[], int> data = mGlobalData.GetData();
                if (data == null)
                {
                    Thread.Sleep(5); // wait for data
                    continue;
                }
                mBufferedProvider.AddSamples(data.Item1, 0, data.Item2);
                // collect positive and negative extremes in this sample screen
                int startIdx = mRenderSkipByte ? 1 : 0;
                while((startIdx+2) <= data.Item2)
                {
                    Array.Copy(data.Item1, startIdx, buffer, 0, 2);
                    short nextData = DecodeByte(buffer);
                    mRenderPos = Math.Max(nextData, mRenderPos);
                    mRenderNeg = Math.Min(nextData, mRenderNeg);
                    mRenderByteCount++;
                    if(mRenderByteCount >= RENDER_SCREEN_SIZE)
                    {
                        AddWavePoint();
                        mRenderPos = mRenderNeg = 0;
                        mRenderByteCount = 0;
                    }
                    startIdx += 2;
                }
                if (startIdx != data.Item2) mRenderSkipByte = true;
                else mRenderSkipByte = false;
                //Debug.WriteLine("[AudioHelper] new data (" + data.Item2 + " bytes)");
                Thread.Sleep(1);
            }
        }

        // set new audio device
        public void SetAudioDevice(int i)
        {
            if (mWaveOut.PlaybackState == PlaybackState.Playing) mWaveOut.Stop();
            mWaveOut.Dispose();
            mWaveOut = new WaveOut
            {
                DeviceNumber = i
            };
            mWaveOut.Init(mVolumeProvider);
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

        // add new point to wave graph
        private void AddWavePoint()
        {
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                mMainWindow.RefreshWaveform(mRenderPos, mRenderNeg);
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

        // set volume
        public void SetVolume(float val)
        {
            if (mVolumeProvider == null || val == mVolumeProvider.Volume) return;
            val = Math.Max(val, 0.0f);
            mVolumeProvider.Volume = val;
        }

        // helper function to decode byte to float
        public short DecodeByte(byte[] array)
        {
            //if (BitConverter.IsLittleEndian)
            //    Array.Reverse(array);
            return BitConverter.ToInt16(array, 0);
        }
    }
}
