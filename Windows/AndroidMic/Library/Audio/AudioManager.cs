using System;
using System.Threading;
using System.Diagnostics;
using System.Collections.Generic;
using System.Windows.Shapes;
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
    // numbers define the applied order of
    // the filters once enabled
    public enum AdvancedFilterType
    {
        FPitchShifter = 0,
        FWhiteNoise = 1,
        FRepeatTrack = 2
    }

    public class AudioManager
    {
        private readonly string TAG = "AudioManager";
        private readonly int MAX_WAIT_TIME = 1000;
        private readonly int MAX_FILTERS_COUNT = 3;

        private WaveOutCapabilities[] devices;
        private int selectedDeviceIdx;
        private IWavePlayer player;
        private readonly int playerDesiredLatency = 100;
        private readonly int playerNumberOfBuffers = 3;
        private readonly WaveFormat format;

        private readonly BufferedWaveProvider bufferedProvider;
        private readonly FilterSpeexDSP speexProvider;
        private FilterRenderer rendererProvider;
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
            speexProvider = new FilterSpeexDSP(bufferedProvider);
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
            speexProvider.Dispose();
            for (int i = 0; i < MAX_FILTERS_COUNT; i++)
            {
                var filter = providerPipeline[i] as IDisposable;
                if (filter != null) filter.Dispose();
            }
            DebugLog("Shutdown");
        }

        // process audio data
        public void Process()
        {
            while (processAllowed)
            {
                var data = sharedBuffer.poll();
                if (data == null || player == null ||
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
            if (deviceIdx < devices.Length)
                selectedDeviceIdx = deviceIdx;
            else
                selectedDeviceIdx = -1;
            // stop playing
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
            // start playing
            player.Init(volumeProvider);
            player.Play();
            DebugLog("SelectAudioDevice: " + deviceIdx);
            AddLog("Device changed to " + ((deviceIdx < 0) ? "Default" : devices[deviceIdx].ProductName));
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
            if (RefreshAudioDevicesEvent != null)
            {
                string[] deviceNames = new string[devices.Length];
                for (int i = 0; i < devices.Length; i++)
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
            ISampleProvider source = speexProvider.ToSampleProvider();
            for (int i = 0; i < MAX_FILTERS_COUNT; i++)
            {
                AdvancedFilterType type = (AdvancedFilterType)i;
                // skip not enabled pipelines
                if (!providerPipelineStates[type]) continue;
                switch (type)
                {
                    case AdvancedFilterType.FPitchShifter:
                        source = providerPipeline[i] = new FilterPitchShifter(source, providerPipeline[i] as FilterPitchShifter);
                        break;
                    case AdvancedFilterType.FWhiteNoise:
                        source = providerPipeline[i] = new FilterWhiteNoise(source, providerPipeline[i] as FilterWhiteNoise);
                        break;
                    case AdvancedFilterType.FRepeatTrack:
                        source = providerPipeline[i] = new FilterRepeatTrack(source, providerPipeline[i] as FilterRepeatTrack);
                        break;
                }
            }
            rendererProvider = new FilterRenderer(source, prev: rendererProvider);
            volumeProvider = new VolumeSampleProvider(rendererProvider)
            {
                Volume = volumeProvider == null ? 1.0f : volumeProvider.Volume
            };
            player.Init(volumeProvider);
            player.Play();
        }

        public void ConfigSpeexDSP(FilterSpeexDSP.ConfigTypes config, ref bool enabled, bool set)
        {
            switch (config)
            {
                case FilterSpeexDSP.ConfigTypes.ConfigDenoise:
                    if (set)
                        speexProvider.EnabledDenoise = enabled;
                    else
                        enabled = speexProvider.EnabledDenoise;
                    break;
                case FilterSpeexDSP.ConfigTypes.ConfigAGC:
                    if (set)
                        speexProvider.EnabledAGC = enabled;
                    else
                        enabled = speexProvider.EnabledAGC;
                    break;
                case FilterSpeexDSP.ConfigTypes.ConfigVAD:
                    if (set)
                        speexProvider.EnabledVAD = enabled;
                    else
                        enabled = speexProvider.EnabledVAD;
                    break;
                case FilterSpeexDSP.ConfigTypes.ConfigEcho:
                    if (set)
                        speexProvider.EnabledEcho = enabled;
                    else
                        enabled = speexProvider.EnabledEcho;
                    break;
            }
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
            switch (type)
            {
                case AdvancedFilterType.FPitchShifter:
                    {
                        FilterPitchShifter.ConfigTypes configType = (FilterPitchShifter.ConfigTypes)config;
                        FilterPitchShifter filter = providerPipeline[(int)type] as FilterPitchShifter;
                        switch (configType)
                        {
                            case FilterPitchShifter.ConfigTypes.ConfigPitch:
                                if (set && filter != null)
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
                case AdvancedFilterType.FWhiteNoise:
                    {
                        FilterWhiteNoise.ConfigTypes configType = (FilterWhiteNoise.ConfigTypes)config;
                        FilterWhiteNoise filter = providerPipeline[(int)type] as FilterWhiteNoise;
                        switch (configType)
                        {
                            case FilterWhiteNoise.ConfigTypes.ConfigStrength:
                                if (set && filter != null)
                                {
                                    filter.Strength = value;
                                }
                                else
                                {
                                    value = filter == null ? 0.0f : filter.Strength;
                                }
                                break;
                        }
                    }
                    break;
                case AdvancedFilterType.FRepeatTrack:
                    {
                        FilterRepeatTrack.ConfigTypes configType = (FilterRepeatTrack.ConfigTypes)config;
                        FilterRepeatTrack filter = providerPipeline[(int)type] as FilterRepeatTrack;
                        switch (configType)
                        {
                            case FilterRepeatTrack.ConfigTypes.ConfigStrength:
                                if (set && filter != null)
                                {
                                    filter.Strength = value;
                                }
                                else
                                {
                                    value = filter == null ? 0.0f : filter.Strength;
                                }
                                break;
                            case FilterRepeatTrack.ConfigTypes.ConfigRepeat:
                                if (set && filter != null)
                                {
                                    filter.Repeat = value != 0.0f;
                                }
                                else
                                {
                                    value = filter == null ? 0.0f : (filter.Repeat ? 1.0f : 0.0f);
                                }
                                break;
                            case FilterRepeatTrack.ConfigTypes.ConfigSelectFile:
                                string openedFileName = filter.SelectFile();
                                if (openedFileName.Length <= 0)
                                {
                                    AddLog("Failed to load track file!");
                                }
                                else
                                {
                                    AddLog("Selected track file: " + openedFileName);
                                }
                                break;
                        }
                    }
                    break;
            }
        }

        public bool IsEnabled(AdvancedFilterType type) => providerPipelineStates[type];

        public void ApplyToCanvas(Canvas c) => rendererProvider.ApplyToCanvas(c);

        public bool ToggleCanvas() => rendererProvider.ToggleCanvas();

        public void SetIndicator(Ellipse e) => speexProvider.SetIndicator(e);


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
