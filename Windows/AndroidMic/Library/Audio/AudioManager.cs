﻿using System;
using System.Threading;
using System.Threading.Tasks;
using System.Diagnostics;
using System.Collections.Generic;
using System.Windows.Shapes;
using System.Windows.Controls;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;
using NAudio.CoreAudioApi;

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
        FRnnoise = 0,
        FPitchShifter = 1,
        FWhiteNoise = 2,
        FRepeatTrack = 3,

        Count
    }

    public class AudioManager
    {
        private readonly string TAG = "AudioManager";
        private readonly int MAX_WAIT_TIME = 1000;

        private MMDeviceCollection devices;
        private int selectedDeviceIdx;
        private IWavePlayer player;
        public volatile int PlayerDesiredLatency = 120; // in milliseconds

        private readonly int AUDIO_SAMPLE_RATE = 16000;
        private readonly int AUDIO_BITS = 16;
        private readonly int AUDIO_CHANNELS = 1;

        private readonly BufferedWaveProvider bufferedProvider;
        private readonly FilterSpeexDSP speexProvider;
        private FilterRenderer rendererProvider;
        private VolumeSampleProvider volumeProvider;
        private readonly ISampleProvider[] providerPipeline;
        private readonly Dictionary<AdvancedFilterType, bool> providerPipelineStates;

        private readonly AudioBuffer sharedBuffer;
        private readonly Task processTask;
        private volatile bool processAllowed;
        private CancellationTokenSource cancellationTokenSource;

        private readonly SynchronizationContext uiContext;

        public event EventHandler<MessageArgs> AddLogEvent;
        public event EventHandler<AudioDevicesArgs> RefreshAudioDevicesEvent;

        public AudioManager(AudioBuffer buffer, SynchronizationContext context)
        {
            sharedBuffer = buffer;
            uiContext = context;

            selectedDeviceIdx = -1;

            var deviceIter = new MMDeviceEnumerator();
            devices = deviceIter.EnumerateAudioEndPoints(DataFlow.Render, DeviceState.Active);
            player = new WasapiOut(deviceIter.GetDefaultAudioEndpoint(DataFlow.Render, Role.Console), AudioClientShareMode.Shared, false, PlayerDesiredLatency);

            providerPipeline = new ISampleProvider[(int)AdvancedFilterType.Count];
            providerPipelineStates = new Dictionary<AdvancedFilterType, bool>();

            for (int i = 0; i < (int)AdvancedFilterType.Count; i++)
                providerPipelineStates[(AdvancedFilterType)i] = false;

            bufferedProvider = new BufferedWaveProvider(new WaveFormat(AUDIO_SAMPLE_RATE, AUDIO_BITS, AUDIO_CHANNELS))
            {
                DiscardOnBufferOverflow = true
            };

            speexProvider = new FilterSpeexDSP(bufferedProvider, uiContext);

            // build filter pipeline
            BuildPipeline();

            processAllowed = true;
            cancellationTokenSource = new CancellationTokenSource();
            processTask = Task.Factory.StartNew(Process, cancellationTokenSource.Token, TaskCreationOptions.LongRunning, TaskScheduler.Default);
            DebugLog("Playing");
        }

        // shutdown manager
        public void Shutdown()
        {
            processAllowed = false;
            if (player.PlaybackState == PlaybackState.Playing) player.Stop();

            if (processTask?.IsCompleted == false)
            {
                cancellationTokenSource.Cancel();
                try
                {
                    processTask?.Wait(MAX_WAIT_TIME);
                }
                catch (OperationCanceledException err)
                {
                    DebugLog("Shutdown -> " + err.Message);
                }
                finally
                {
                    processTask?.Dispose();
                }
            }

            player.Dispose();
            speexProvider.Dispose();

            for (int i = 0; i < (int)AdvancedFilterType.Count; i++)
            {
                if (providerPipeline[i] is IDisposable filter) filter.Dispose();
            }
            DebugLog("Shutdown");
        }

        // process audio data
        public async void Process()
        {
            while (processAllowed)
            {
                if (cancellationTokenSource.IsCancellationRequested)
                {
                    break;
                }

                if (sharedBuffer.IsEmpty() || player == null ||
                    player.PlaybackState != PlaybackState.Playing)
                {
                    await Task.Delay(5);
                    continue;
                }

                var result = await sharedBuffer.OpenReadRegion(Streaming.Streamer.BUFFER_SIZE);
                var count = result.Item1;
                var offset = result.Item2;

                if (bufferedProvider.BufferedDuration.TotalMilliseconds <= PlayerDesiredLatency)
                {
                    bufferedProvider.AddSamples(sharedBuffer.Buffer, offset, count);
                }

                sharedBuffer.CloseReadRegion(count);
            }
        }

        // select audio device by UI
        public void SelectAudioDevice(int deviceIdx)
        {
            if (deviceIdx < devices.Count)
                selectedDeviceIdx = deviceIdx;
            else
                selectedDeviceIdx = -1;

            // stop playing
            player.Stop();
            player.Dispose();

            // create new player
            var deviceIter = new MMDeviceEnumerator();
            var device = selectedDeviceIdx < 0 ? deviceIter.GetDefaultAudioEndpoint(DataFlow.Render, Role.Console) : devices[selectedDeviceIdx];
            player = new WasapiOut(device, AudioClientShareMode.Shared, false, PlayerDesiredLatency);
            bufferedProvider.ClearBuffer();

            // start playing
            player.Init(volumeProvider);
            player.Play();
            DebugLog("SelectAudioDevice: " + deviceIdx);
            AddLog("Device changed to " + ((deviceIdx < 0) ? "Default" : devices[deviceIdx].FriendlyName));

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
            var deviceIter = new MMDeviceEnumerator();
            devices = deviceIter.EnumerateAudioEndPoints(DataFlow.Render, DeviceState.Active);

            if (RefreshAudioDevicesEvent != null)
            {
                string[] deviceNames = new string[devices.Count];

                for (int i = 0; i < devices.Count; i++)
                    deviceNames[i] = devices[i].FriendlyName;

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
            var deviceIter = new MMDeviceEnumerator();
            var device = selectedDeviceIdx < 0 ? deviceIter.GetDefaultAudioEndpoint(DataFlow.Render, Role.Console) : devices[selectedDeviceIdx];
            player = new WasapiOut(device, AudioClientShareMode.Shared, false, PlayerDesiredLatency);

            bufferedProvider.ClearBuffer();

            ISampleProvider source = speexProvider.ToSampleProvider();

            for (int i = 0; i < (int)AdvancedFilterType.Count; i++)
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
                    case AdvancedFilterType.FRnnoise:
                        source = providerPipeline[i] = new WdlResamplingSampleProvider(new FilterRnnoise(source, providerPipeline[i] as FilterRnnoise), AUDIO_SAMPLE_RATE);
                        break;
                }
            }

            rendererProvider = new FilterRenderer(source, uiContext, prev: rendererProvider);

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
