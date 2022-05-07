using System;
using System.Threading;
using System.Diagnostics;
using System.Collections.Generic;
using System.Windows.Controls;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;

namespace AndroidMic.Audio
{
    public class AudioDevicesArgs : EventArgs
    {
        public string[] Devices { get; set; }
        public int SelectedIdx { get; set; }
    }

    // supported filters
    public enum AdvancedFilterType
    {
        FPitchShifter = 0
    }

    public class AudioManager
    {
        private readonly string TAG = "AudioManager";
        private readonly int MAX_WAIT_TIME = 1000;
        private readonly int MAX_FILTERS_COUNT = 1;

        private WaveOutCapabilities[] devices;
        private int selectedDeviceIdx;
        private IWavePlayer player;
        private readonly int playerDesiredLatency = 100;
        private readonly int playerNumberOfBuffers = 3;
        private readonly WaveFormat format;

        private readonly BufferedWaveProvider bufferedProvider;
        private readonly FilterRenderer rendererProvider;
        private VolumeSampleProvider volumeProvider;
        private readonly ISampleProvider[] providerPipeline;
        private readonly Dictionary<AdvancedFilterType, bool> providerPipelineStates;

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
                DeviceNumber = selectedDeviceIdx,
                DesiredLatency = playerDesiredLatency,
                NumberOfBuffers = playerNumberOfBuffers
            };
            providerPipeline = new ISampleProvider[MAX_FILTERS_COUNT];
            providerPipelineStates = new Dictionary<AdvancedFilterType, bool>();
            for (int i = 0; i < MAX_FILTERS_COUNT; i++)
                providerPipelineStates[(AdvancedFilterType)i] = false;
            bufferedProvider = new BufferedWaveProvider(format)
            {
                DiscardOnBufferOverflow = true
            };
            rendererProvider = new FilterRenderer(bufferedProvider.ToSampleProvider());
            // build filter pipeline
            BuildPipeline();
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
            player.Init(volumeProvider);
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

        // build audio filter pipeline
        private void BuildPipeline()
        {
            player.Stop();
            player.Dispose();
            // create new player
            player = new WaveOut
            {
                DeviceNumber = selectedDeviceIdx,
                DesiredLatency = playerDesiredLatency,
                NumberOfBuffers = playerNumberOfBuffers
            };
            bufferedProvider.ClearBuffer();
            ISampleProvider source = rendererProvider;
            for (int i = 0; i < MAX_FILTERS_COUNT; i++)
            {
                AdvancedFilterType type = (AdvancedFilterType)i;
                // skip not enabled pipelines
                if (!providerPipelineStates[type]) continue;
                switch(type)
                {
                    case AdvancedFilterType.FPitchShifter:
                        source = providerPipeline[i] = new FilterPitchShifter(source, providerPipeline[i] as FilterPitchShifter);
                        break;
                }
            }
            volumeProvider = new VolumeSampleProvider(source)
            {
                Volume = volumeProvider == null ? 1.0f : volumeProvider.Volume
            };
            player.Init(volumeProvider);
            player.Play();
        }

        // update a filter state
        public void UpdatePipelineFilter(AdvancedFilterType type, bool enabled)
        {
            bool shouldUpdate = providerPipelineStates[type] != enabled;
            providerPipelineStates[type] = enabled;
            if (shouldUpdate) BuildPipeline();
        }

        // config a filter
        public void PipelineFilterConfig(AdvancedFilterType type, int config, ref float value, bool set)
        {
            switch(type)
            {
                case AdvancedFilterType.FPitchShifter:
                    {
                        FilterPitchShifter.ConfigTypes configType = (FilterPitchShifter.ConfigTypes)config;
                        FilterPitchShifter filter = providerPipeline[(int)type] as FilterPitchShifter;
                        switch (configType)
                        {
                            case FilterPitchShifter.ConfigTypes.ConfigPitch:
                                if(set && filter != null)
                                {
                                    filter.PitchShift = value;
                                }
                                else
                                {
                                    value = filter == null ? 1.0f : filter.PitchShift;
                                }
                                break;
                        }
                    }
                    break;
            }
        }

        public bool IsEnabled(AdvancedFilterType type) => providerPipelineStates[type];

        public void ApplyToCanvas(Canvas c) => rendererProvider.ApplyToCanvas(c);

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
