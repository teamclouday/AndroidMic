using System;
using System.Windows;
using System.Windows.Media;
using System.Windows.Shapes;
using System.Runtime.InteropServices;
using NAudio.Wave;
using NAudio.Utils;
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

        private readonly int FRAME_SIZE = 640; // 40ms (0.04 * 16000)
        private readonly int FRAME_SIZE_BYTES = 640 * 2; // in bytes
        private readonly int FILTER_LEN = 512; // or 100

        // configs specifically chosen for this app
        private readonly int VAD_PROB_START = 85;
        private readonly int VAD_PROB_CONTINUE = 70;
        private readonly int ECHO_SUPPRESS = -60; // in dB
        private readonly int ECHO_SUPPRESS_ACTIVE = -60; // in dB
        private readonly int NOISE_SUPPRESS = -25; // in dB
        private readonly int AGC_LEVEL = 24000;

        private readonly short[] audioBuffer;
        //private readonly short[] echoPlayBuffer;
        //private readonly short[] echoOutBuffer;
        //private readonly byte[] echoPlayerCircBufferSource;
        //private readonly CircularBuffer echoPlayerCircBuffer;
        //private bool enableCircBuffer;

        private readonly IWaveProvider source;
        private Ellipse indicator;
        private readonly Brush indicatorOn;
        private readonly Brush indicatorOff;

        private readonly IntPtr SpeexPreprocessState;
        private readonly IntPtr SpeexEchoState;
        private readonly IntPtr StateUpdate;

        public FilterSpeexDSP(IWaveProvider source)
        {
            this.source = source;
            audioBuffer = new short[FRAME_SIZE];
            //echoPlayBuffer = new short[FRAME_SIZE];
            //echoOutBuffer = new short[FRAME_SIZE];
            //echoPlayerCircBufferSource = new byte[FRAME_SIZE_B];
            //echoPlayerCircBuffer = new CircularBuffer(WaveFormat.AverageBytesPerSecond * 5);
            //enableCircBuffer = false;
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
            while(toRead > 0)
            {
                // copy buffer
                int nextRead = Math.Min(toRead, FRAME_SIZE_BYTES); // in bytes
                Buffer.BlockCopy(buffer, offset, audioBuffer, 0, nextRead);
                // clear audio buffer remaining shorts (bytes / 2)
                if (nextRead < FRAME_SIZE_BYTES) Array.Clear(audioBuffer, nextRead / 2, (FRAME_SIZE_BYTES - nextRead) / 2);
                // process audio
                InSpeech = SpeexPreprocess.speex_preprocess_run(SpeexPreprocessState, audioBuffer) == 1;
                // copy back
                Buffer.BlockCopy(audioBuffer, 0, buffer, offset, nextRead);
                // update samples to read
                toRead -= nextRead;
                offset += nextRead;
            }
            // check if VAD enabled
            InSpeech = InSpeech && EnabledVAD;
            // update indicator
            Application.Current.Dispatcher.Invoke(new Action(() =>
            {
                UpdateIndicator();
            }));
            return samplesRead;
        }

        // dispose object
        public void Dispose()
        {
            SpeexEcho.speex_echo_state_destroy(SpeexEchoState);
            SpeexPreprocess.speex_preprocess_state_destroy(SpeexPreprocessState);
            Marshal.FreeHGlobal(StateUpdate);
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
            indicator.Fill = InSpeech ? indicatorOn : indicatorOff;
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
                if(SpeexPreprocessState != null)
                {
                    Marshal.WriteInt32(StateUpdate, _EnabledDenoise ? 1 : 2);
                    // Turns denoising on(1) or off(2)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_DENOISE,
                        StateUpdate);
                    if(_EnabledDenoise)
                    {
                        // Set maximum attenuation of the noise in dB (negative number)
                        Marshal.WriteInt32(StateUpdate, NOISE_SUPPRESS);
                        SpeexPreprocess.speex_preprocess_ctl(
                            SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_NOISE_SUPPRESS,
                            StateUpdate);
                    }
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
                if(SpeexPreprocessState != null)
                {
                    Marshal.WriteInt32(StateUpdate, _EnabledAGC ? 1 : 2);
                    // Turns automatic gain control (AGC) on(1) or off(2)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_AGC,
                        StateUpdate);
                    if(_EnabledAGC)
                    {
                        // Set preprocessor Automatic Gain Control level
                        Marshal.WriteInt32(StateUpdate, AGC_LEVEL);
                        SpeexPreprocess.speex_preprocess_ctl(
                            SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_AGC_TARGET,
                            StateUpdate);
                    }
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
                if (SpeexPreprocessState != null)
                {
                    Marshal.WriteInt32(StateUpdate, _EnabledVAD ? 1 : 2);
                    // Turns voice activity detector (VAD) on(1) or off(2)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_VAD,
                        StateUpdate);
                    if(_EnabledVAD)
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
                if (SpeexPreprocessState != null)
                {
                    // Set the associated echo canceller for residual echo suppression
                    // (pointer or NULL for no residual echo suppression)
                    SpeexPreprocess.speex_preprocess_ctl(
                        SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_STATE,
                        _EnabledEcho ? SpeexEchoState : IntPtr.Zero);
                    if(_EnabledEcho)
                    {
                        Marshal.WriteInt32(StateUpdate, ECHO_SUPPRESS);
                        SpeexPreprocess.speex_preprocess_ctl(
                            SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_SUPPRESS,
                            StateUpdate);
                        Marshal.WriteInt32(StateUpdate, ECHO_SUPPRESS_ACTIVE);
                        SpeexPreprocess.speex_preprocess_ctl(
                            SpeexPreprocessState, SpeexPreprocess.SPEEX_PREPROCESS_SET_ECHO_SUPPRESS_ACTIVE,
                            StateUpdate);
                    }
                }
            }
        }

        public bool InSpeech { get; private set; }
    }
}
