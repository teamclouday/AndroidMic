using System;
using System.Threading;
using System.Diagnostics;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;

namespace AndroidMic.Audio
{
    public class AudioDevicesArgs : EventArgs
    {
        public string[] Devices { get; set; }
        public int SelectedIdx { get; set; }
    }

    public class AudioManager
    {
        private readonly string TAG = "AudioManager";
        private readonly int MAX_WAIT_TIME = 1000;

        private WaveOutCapabilities[] devices;
        private int selectedDeviceIdx;
        private IWavePlayer player;
        private readonly int playerDesiredLatency = 50;
        private readonly int playerNumberOfBuffers = 3;
        private readonly WaveFormat format;

        private readonly BufferedWaveProvider bufferedProvider;
        public readonly FilterRenderer RendererProvider;
        private readonly VolumeSampleProvider volumeProvider;

        private readonly AudioBuffer sharedBuffer;
        private readonly Thread processThread;
        private volatile bool processAllowed;

        public event EventHandler<MessageArgs> AddLogEvent;
        public event EventHandler<AudioDevicesArgs> RefreshAudioDevicesEvent;

        public AudioManager(AudioBuffer buffer)
        {
            sharedBuffer = buffer;
            selectedDeviceIdx = -1;
            format = new WaveFormat(16000, 16, 1); // sample rate, bits, channels
            player = new WaveOut
            {
                DeviceNumber = -1,
                DesiredLatency = playerDesiredLatency,
                NumberOfBuffers = playerNumberOfBuffers
            };
            bufferedProvider = new BufferedWaveProvider(format)
            {
                DiscardOnBufferOverflow = true
            };
            RendererProvider = new FilterRenderer(bufferedProvider.ToSampleProvider());
            volumeProvider = new VolumeSampleProvider(RendererProvider);
            player.Init(volumeProvider);
            player.Play();
            processAllowed = true;
            processThread = new Thread(new ThreadStart(Process));
            processThread.Start();
            DebugLog("Playing");
        }

        // shutdown manager
        public void Shutdown()
        {
            if (player.PlaybackState == PlaybackState.Playing) player.Stop();
            if (processThread != null && processThread.IsAlive)
            {
                processAllowed = false;
                processThread.Join(MAX_WAIT_TIME);
            }
            player.Dispose();
            DebugLog("Shutdown");
        }

        // process audio data
        public void Process()
        {
            while(processAllowed)
            {
                var data = sharedBuffer.poll();
                if(data == null || player == null ||
                    player.PlaybackState != PlaybackState.Playing)
                {
                    Thread.Sleep(5);
                    continue;
                }
                bufferedProvider.AddSamples(data, 0, data.Length);
            }
        }

        // select audio device by UI
        public void SelectAudioDevice(int deviceIdx)
        {
            // stop playing
            player.Stop();
            player.Dispose();
            // create new player
            player = new WaveOut
            {
                DeviceNumber = deviceIdx,
                DesiredLatency = playerDesiredLatency,
                NumberOfBuffers = playerNumberOfBuffers
            };
            bufferedProvider.ClearBuffer();
            // start playing
            player.Init(bufferedProvider);
            player.Play();
            DebugLog("SelectAudioDevice: " + deviceIdx);
            AddLog("Device changed to " + ((deviceIdx < 0) ? "Default" : devices[deviceIdx].ProductName));
            selectedDeviceIdx = deviceIdx;
            RefreshAudioDevices();
        }

        // adjust volume
        public void SetVolume(float val)
        {
            volumeProvider.Volume = Math.Min(Math.Max(val, 0.0f), 5.0f);
        }

        // refresh audio devices
        public void RefreshAudioDevices()
        {
            if (devices == null || WaveOut.DeviceCount != devices.Length)
                devices = new WaveOutCapabilities[WaveOut.DeviceCount];
            for (int i = 0; i < devices.Length; i++)
                devices[i] = WaveOut.GetCapabilities(i);
            if(RefreshAudioDevicesEvent != null)
            {
                string[] deviceNames = new string[devices.Length];
                for(int i = 0; i < devices.Length; i++)
                    deviceNames[i] = devices[i].ProductName;
                RefreshAudioDevicesEvent?.Invoke(this, new AudioDevicesArgs
                {
                    Devices = deviceNames,
                    SelectedIdx = selectedDeviceIdx
                });
            }
        }

        // add log message to main window
        private void AddLog(string message)
        {
            AddLogEvent?.Invoke(this, new MessageArgs()
            {
                Message = "[Audio Manager]\n" + message + "\n"
            });
        }

        // debug log
        private void DebugLog(string message)
        {
            Debug.WriteLine(string.Format("[{0}] {1}", TAG, message));
        }
    }
}
