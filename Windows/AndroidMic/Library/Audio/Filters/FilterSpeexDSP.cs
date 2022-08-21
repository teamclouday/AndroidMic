using System;
using System.Threading;
using System.Windows;
using System.Windows.Media;
using System.Windows.Shapes;
using System.Runtime.InteropServices;
using NAudio.Utils;
using NAudio.Wave;
using libspeexdsp;

namespace AndroidMic.Audio
{
    // use libspeexdsp preprocessor
    // reference: iaxclient/lib/audio_encode.c
    public class FilterSpeexDSP : IWaveProvider, IDisposable
    {
        public enum ConfigTypes
        {
            ConfigDenoise = 0,
            ConfigAGC = 1,
            ConfigEcho = 2,
            ConfigVAD = 3
        }

        private readonly int FRAME_SIZE = 800; // 50ms (0.05 * 16000)
        private readonly int FRAME_SIZE_BYTES = 800 * 2; // in bytes
        private readonly int FILTER_LEN = 3200; // 100-500 ms (0.2 * 16000)

        // configs specifically chosen for this app
        private readonly int VAD_PROB_START = 85;
        private readonly int VAD_PROB_CONTINUE = 65;
        private readonly int ECHO_SUPPRESS = -45; // in dB
        private readonly int ECHO_SUPPRESS_ACTIVE = -45; // in dB
        private readonly int NOISE_SUPPRESS = -25; // in dB
        private readonly int AGC_LEVEL = 24000;

        private readonly byte[] audioBuffer;
        private readonly byte[] echoPlayBuffer;
        private readonly CircularBuffer loopbackPlayBuffer;
        private readonly object loopbackPlayBufferLock;

        private readonly IWaveProvider source;

        // capture variables for echo cancellation play buffer
        private WasapiLoopbackCapture loopbackCapture;

        private Ellipse indicator;
        private readonly Brush indicatorOn;
        private readonly Brush indicatorOff;
        private bool indicatorInSpeech;

        private readonly IntPtr SpeexPreprocessState;
        private readonly IntPtr StateUpdate;
        private IntPtr SpeexEchoState;

        private readonly SynchronizationContext uiContext;

        public FilterSpeexDSP(IWaveProvider source, SynchronizationContext uiContext)
        {
            this.source = source;
            this.uiContext = uiContext;

            audioBuffer = new byte[FRAME_SIZE_BYTES];
            echoPlayBuffer = new byte[FRAME_SIZE_BYTES];

            loopbackPlayBuffer = new CircularBuffer(WaveFormat.AverageBytesPerSecond * 5); // 5 seconds recording
            loopbackPlayBufferLock = new object();

            SpeexPreprocessState = SpeexPreprocess.speex_preprocess_state_init(FRAME_SIZE, WaveFormat.SampleRate);
            SpeexEchoState = SpeexEcho.speex_echo_state_init(FRAME_SIZE, FILTER_LEN);

            StateUpdate = Marshal.AllocHGlobal(sizeof(int));
            _EnabledDenoise = false;
            _EnabledAGC = false;
            _EnabledVAD = false;
            _EnabledEcho = false;

            var converter = new BrushConverter();
            indicatorOn = converter.ConvertFromString("#FFFF9494") as SolidColorBrush;
            indicatorOff = converter.ConvertFromString("#94fffa") as SolidColorBrush;
        }

        // read call
        public int Read(byte[] buffer, int offset, int sampleCount)
        {
            int samplesRead = source.Read(buffer, offset, sampleCount);
            // skip if no filters are enabled
            if (!(EnabledDenoise || EnabledAGC || EnabledEcho || EnabledVAD)) return samplesRead;
            // else do audio processing
            int toRead = samplesRead; // in bytes
            while (toRead > 0)
            {
                // copy buffer
                int nextRead = Math.Min(toRead, FRAME_SIZE_BYTES); // in bytes
                Buffer.BlockCopy(buffer, offset, audioBuffer, 0, nextRead);
                if (nextRead < FRAME_SIZE_BYTES) Array.Clear(audioBuffer, nextRead, FRAME_SIZE_BYTES - nextRead);
                // do echo cancellation first if enabled
                if (EnabledEcho)
                {
                    // prepare recorded play buffer
                    lock (loopbackPlayBufferLock)
                    {
                        int bytesRead = loopbackPlayBuffer.Read(echoPlayBuffer, 0, echoPlayBuffer.Length);
                        if (bytesRead < echoPlayBuffer.Length) Array.Clear(echoPlayBuffer, bytesRead, echoPlayBuffer.Length - bytesRead);
                    }
                    // echo cancellation
                    SpeexEcho.speex_echo_cancellation(SpeexEchoState, audioBuffer, echoPlayBuffer, audioBuffer);
                }
                // process audio
                indicatorInSpeech = SpeexPreprocess.speex_preprocess_run(SpeexPreprocessState, audioBuffer) == 1;
                // copy back
                Buffer.BlockCopy(audioBuffer, 0, buffer, offset, nextRead);
                // update samples to read
                toRead -= nextRead;
                offset += nextRead;
            }
            // check if VAD enabled
            indicatorInSpeech = indicatorInSpeech && EnabledVAD;
            // update indicator
            uiContext?.Post(delegate
            {
                UpdateIndicator();
            }, null);
            return samplesRead;
        }

        // dispose object
        public void Dispose()
        {
            SpeexEcho.speex_echo_state_destroy(SpeexEchoState);
            SpeexPreprocess.speex_preprocess_state_destroy(SpeexPreprocessState);
            Marshal.FreeHGlobal(StateUpdate);
            StopCapture();
        }

        // set indicator
        public void SetIndicator(Ellipse e)
        {
            indicator = e;
        }

        // update indicator
        private void UpdateIndicator()
        {
            if (indicator == null) return;
            indicator.Fill = indicatorInSpeech ? indicatorOn : indicatorOff;
        }

        // start PC speaker capture
        private void StartCapture()
        {
            StopCapture();
            loopbackCapture = new WasapiLoopbackCapture();
            loopbackCapture.DataAvailable += (s, e) =>
            {
                // record into loopback buffer
                lock (loopbackPlayBufferLock)
                {
                    loopbackPlayBuffer.Write(e.Buffer, 0, e.BytesRecorded);
                }
            };
            loopbackCapture.RecordingStopped += (s, a) =>
            {
                loopbackCapture.Dispose();
            };
            loopbackCapture.WaveFormat = WaveFormat;
            loopbackCapture.StartRecording();
        }

        // stop PC speaker capture
        private void StopCapture()
        {
            loopbackCapture?.StopRecording();
        }

        public WaveFormat WaveFormat => source.WaveFormat;

        private bool _EnabledDenoise;
        public bool EnabledDenoise
        {
            get
            {
                return _EnabledDenoise;
            }
            set
            {
                _EnabledDenoise = value;
                // Turns denoising on(1) or off(2)
                Marshal.WriteInt32(StateUpdate, _EnabledDenoise ? 1 : 2);
                SpeexPreprocess.speex_preprocess_ctl(
                    SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_DENOISE,
                    StateUpdate);
                if (_EnabledDenoise)
                {
                    // Set maximum attenuation of the noise in dB (negative number)
                    Marshal.WriteInt32(StateUpdate, NOISE_SUPPRESS);
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_NOISE_SUPPRESS,
                        StateUpdate);
                }
            }
        }

        private bool _EnabledAGC;
        public bool EnabledAGC
        {
            get
            {
                return _EnabledAGC;
            }
            set
            {
                _EnabledAGC = value;
                // Turns automatic gain control (AGC) on(1) or off(2)
                Marshal.WriteInt32(StateUpdate, _EnabledAGC ? 1 : 2);
                SpeexPreprocess.speex_preprocess_ctl(
                    SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_AGC,
                    StateUpdate);
                if (_EnabledAGC)
                {
                    // Set preprocessor Automatic Gain Control level
                    Marshal.WriteInt32(StateUpdate, AGC_LEVEL);
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_AGC_TARGET,
                        StateUpdate);
                }
            }
        }

        private bool _EnabledVAD;
        public bool EnabledVAD
        {
            get
            {
                return _EnabledVAD;
            }
            set
            {
                _EnabledVAD = value;
                // Turns voice activity detector (VAD) on(1) or off(2)
                Marshal.WriteInt32(StateUpdate, _EnabledVAD ? 1 : 2);
                SpeexPreprocess.speex_preprocess_ctl(
                    SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_VAD,
                    StateUpdate);
                if (_EnabledVAD)
                {
                    // Set probability required for the VAD to go from silence to voice
                    Marshal.WriteInt32(StateUpdate, VAD_PROB_START);
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_PROB_START,
                        StateUpdate);
                    // Set probability required for the VAD to stay in the voice state (integer percent)
                    Marshal.WriteInt32(StateUpdate, VAD_PROB_CONTINUE);
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_PROB_CONTINUE,
                        StateUpdate);
                }
                else
                {
                    indicatorInSpeech = false;
                    // update indicator
                    uiContext?.Post(delegate
                    {
                        UpdateIndicator();
                    }, null);
                }
            }
        }

        private bool _EnabledEcho;
        public bool EnabledEcho
        {
            get
            {
                return _EnabledEcho;
            }
            set
            {
                _EnabledEcho = value;
                // Set the associated echo canceller for residual echo suppression
                // (pointer or NULL for no residual echo suppression)
                SpeexPreprocess.speex_preprocess_ctl(
                    SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_STATE,
                    _EnabledEcho ? SpeexEchoState : IntPtr.Zero);
                if (_EnabledEcho)
                {
                    // Set sampling rate
                    Marshal.WriteInt32(StateUpdate, WaveFormat.SampleRate);
                    SpeexEcho.speex_echo_ctl(
                        SpeexEchoState, SpeexEcho.SPEEX_ECHO_SET_SAMPLING_RATE,
                        StateUpdate);
                    // Set maximum attenuation of the residual echo in dB (negative number)
                    Marshal.WriteInt32(StateUpdate, ECHO_SUPPRESS);
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_SUPPRESS,
                        StateUpdate);
                    // Set maximum attenuation of the residual echo in dB when near end is active (negative number)
                    Marshal.WriteInt32(StateUpdate, ECHO_SUPPRESS_ACTIVE);
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_SUPPRESS_ACTIVE,
                        StateUpdate);
                    // Start capturing for play buffer
                    StartCapture();
                }
                else
                {
                    StopCapture();
                    // reset echo state
                    SpeexEcho.speex_echo_state_destroy(SpeexEchoState);
                    SpeexEchoState = SpeexEcho.speex_echo_state_init(FRAME_SIZE, FILTER_LEN);
                }
            }
        }
    }
}
