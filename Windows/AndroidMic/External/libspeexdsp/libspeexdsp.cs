using System;
using System.Runtime.InteropServices;

namespace libspeexdsp
{
    // C# binding of libspeexdsp speex_echo.h header
    public class SpeexEcho
    {
        /** Obtain frame size used by the AEC */
        public static int SPEEX_ECHO_GET_FRAME_SIZE = 3;

        /** Set sampling rate */
        public static int SPEEX_ECHO_SET_SAMPLING_RATE = 24;
        /** Get sampling rate */
        public static int SPEEX_ECHO_GET_SAMPLING_RATE = 25;

        /* Can't set window sizes */
        /** Get size of impulse response (int32) */
        public static int SPEEX_ECHO_GET_IMPULSE_RESPONSE_SIZE = 27;

        /* Can't set window content */
        /** Get impulse response (int32[]) */
        public static int SPEEX_ECHO_GET_IMPULSE_RESPONSE = 29;

        /** Creates a new echo canceller state
         * @param frame_size Number of samples to process at one time (should correspond to 10-20 ms)
         * @param filter_length Number of samples of echo to cancel (should generally correspond to 100-500 ms)
         * @return Newly-created echo canceller state
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr speex_echo_state_init(int frame_size, int filter_length);

        /** Creates a new multi-channel echo canceller state
         * @param frame_size Number of samples to process at one time (should correspond to 10-20 ms)
         * @param filter_length Number of samples of echo to cancel (should generally correspond to 100-500 ms)
         * @param nb_mic Number of microphone channels
         * @param nb_speakers Number of speaker channels
         * @return Newly-created echo canceller state
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr speex_echo_state_init_mc(int frame_size, int filter_length, int nb_mic, int nb_speakers);

        /** Destroys an echo canceller state
         * @param st Echo canceller state
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_echo_state_destroy(IntPtr st);

        /** Performs echo cancellation a frame, based on the audio sent to the speaker (no delay is added
         * to playback in this form)
         *
         * @param st Echo canceller state
         * @param rec Signal from the microphone (near end + far end echo)
         * @param play Signal played to the speaker (received from far end)
         * @param outt Returns near-end signal with echo removed
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_echo_cancellation(IntPtr st, short[] rec, short[] play, short[] outt);

        // byte array version, as long as audio data is from 16 bit stream
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_echo_cancellation(IntPtr st, byte[] rec, byte[] play, byte[] outt);

        /** Perform echo cancellation using internal playback buffer, which is delayed by two frames
         * to account for the delay introduced by most soundcards (but it could be off!)
         * @param st Echo canceller state
         * @param rec Signal from the microphone (near end + far end echo)
         * @param outt Returns near-end signal with echo removed
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_echo_capture(IntPtr st, short[] rec, short[] outt);

        /** Let the echo canceller know that a frame was just queued to the soundcard
         * @param st Echo canceller state
         * @param play Signal played to the speaker (received from far end)
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_echo_playback(IntPtr st, short[] play);

        /** Reset the echo canceller to its original state
         * @param st Echo canceller state
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_echo_state_reset(IntPtr st);

        /** Used like the ioctl function to control the echo canceller parameters
         *
         * @param st Echo canceller state
         * @param request ioctl-type request (one of the SPEEX_ECHO_* macros)
         * @param ptr Data exchanged to-from function
         * @return 0 if no error, -1 if request in unknown
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern int speex_echo_ctl(IntPtr st, int request, IntPtr ptr);

        /** Create a state for the channel decorrelation algorithm
            This is useful for multi-channel echo cancellation only
         * @param rate Sampling rate
         * @param channels Number of channels (it's a bit pointless if you don't have at least 2)
         * @param frame_size Size of the frame to process at ones (counting samples *per* channel)
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr speex_decorrelate_new(int rate, int channels, int frame_size);

        /** Remove correlation between the channels by modifying the phase and possibly
            adding noise in a way that is not (or little) perceptible.
         * @param st Decorrelator state
         * @param inn Input audio in interleaved format
         * @param outt Result of the decorrelation (out *may* alias in)
         * @param strength How much alteration of the audio to apply from 0 to 100.
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_decorrelate(IntPtr st, short[] inn, short[] outt, int strength);

        /** Destroy a Decorrelation state
         * @param st State to destroy
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_decorrelate_destroy(IntPtr st);
    }

    // C# binding of libspeexdsp speex_preprocess.h header
    public class SpeexPreprocess
    {
        /** Creates a new preprocessing state. You MUST create one state per channel processed.
         * @param frame_size Number of samples to process at one time (should correspond to 10-20 ms). Must be
         * the same value as that used for the echo canceller for residual echo cancellation to work.
         * @param sampling_rate Sampling rate used for the input.
         * @return Newly created preprocessor state
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr speex_preprocess_state_init(int frame_size, int sampling_rate);

        /** Destroys a preprocessor state
         * @param st Preprocessor state to destroy
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_preprocess_state_destroy(IntPtr st);

        /** Preprocess a frame
         * @param st Preprocessor state
         * @param x Audio sample vector (in and out). Must be same size as specified in speex_preprocess_state_init().
         * @return Bool value for voice activity (1 for speech, 0 for noise/silence), ONLY if VAD turned on.
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern int speex_preprocess_run(IntPtr st, short[] x);

        // byte array version, as long as audio data is from 16 bit stream
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern int speex_preprocess_run(IntPtr st, byte[] x);

        /** Update preprocessor state, but do not compute the output
         * @param st Preprocessor state
         * @param x Audio sample vector (in only). Must be same size as specified in speex_preprocess_state_init().
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern void speex_preprocess_estimate_update(IntPtr st, short[] x);

        /** Used like the ioctl function to control the preprocessor parameters
         * @param st Preprocessor state
         * @param request ioctl-type request (one of the SPEEX_PREPROCESS_* macros)
         * @param ptr Data exchanged to-from function
         * @return 0 if no error, -1 if request in unknown
        */
        [DllImport("libspeexdsp.dll", CallingConvention = CallingConvention.Cdecl)]
        public static extern int speex_preprocess_ctl(IntPtr st, int request, IntPtr ptr);

        /** Set preprocessor denoiser state */
        public static int SPEEX_PREPROCESS_SET_DENOISE = 0;
        /** Get preprocessor denoiser state */
        public static int SPEEX_PREPROCESS_GET_DENOISE = 1;

        /** Set preprocessor Automatic Gain Control state */
        public static int SPEEX_PREPROCESS_SET_AGC = 2;
        /** Get preprocessor Automatic Gain Control state */
        public static int SPEEX_PREPROCESS_GET_AGC = 3;

        /** Set preprocessor Voice Activity Detection state */
        public static int SPEEX_PREPROCESS_SET_VAD = 4;
        /** Get preprocessor Voice Activity Detection state */
        public static int SPEEX_PREPROCESS_GET_VAD = 5;

        /** Set preprocessor Automatic Gain Control level (float) */
        public static int SPEEX_PREPROCESS_SET_AGC_LEVEL = 6;
        /** Get preprocessor Automatic Gain Control level (float) */
        public static int SPEEX_PREPROCESS_GET_AGC_LEVEL = 7;

        /** Set preprocessor dereverb state */
        public static int SPEEX_PREPROCESS_SET_DEREVERB = 8;
        /** Get preprocessor dereverb state */
        public static int SPEEX_PREPROCESS_GET_DEREVERB = 9;

        /** Set preprocessor dereverb level */
        public static int SPEEX_PREPROCESS_SET_DEREVERB_LEVEL = 10;
        /** Get preprocessor dereverb level */
        public static int SPEEX_PREPROCESS_GET_DEREVERB_LEVEL = 11;

        /** Set preprocessor dereverb decay */
        public static int SPEEX_PREPROCESS_SET_DEREVERB_DECAY = 12;
        /** Get preprocessor dereverb decay */
        public static int SPEEX_PREPROCESS_GET_DEREVERB_DECAY = 13;

        /** Set probability required for the VAD to go from silence to voice */
        public static int SPEEX_PREPROCESS_SET_PROB_START = 14;
        /** Get probability required for the VAD to go from silence to voice */
        public static int SPEEX_PREPROCESS_GET_PROB_START = 15;

        /** Set probability required for the VAD to stay in the voice state (integer percent) */
        public static int SPEEX_PREPROCESS_SET_PROB_CONTINUE = 16;
        /** Get probability required for the VAD to stay in the voice state (integer percent) */
        public static int SPEEX_PREPROCESS_GET_PROB_CONTINUE = 17;

        /** Set maximum attenuation of the noise in dB (negative number) */
        public static int SPEEX_PREPROCESS_SET_NOISE_SUPPRESS = 18;
        /** Get maximum attenuation of the noise in dB (negative number) */
        public static int SPEEX_PREPROCESS_GET_NOISE_SUPPRESS = 19;

        /** Set maximum attenuation of the residual echo in dB (negative number) */
        public static int SPEEX_PREPROCESS_SET_ECHO_SUPPRESS = 20;
        /** Get maximum attenuation of the residual echo in dB (negative number) */
        public static int SPEEX_PREPROCESS_GET_ECHO_SUPPRESS = 21;

        /** Set maximum attenuation of the residual echo in dB when near end is active (negative number) */
        public static int SPEEX_PREPROCESS_SET_ECHO_SUPPRESS_ACTIVE = 22;
        /** Get maximum attenuation of the residual echo in dB when near end is active (negative number) */
        public static int SPEEX_PREPROCESS_GET_ECHO_SUPPRESS_ACTIVE = 23;

        /** Set the corresponding echo canceller state so that residual echo suppression can be performed (NULL for no residual echo suppression) */
        public static int SPEEX_PREPROCESS_SET_ECHO_STATE = 24;
        /** Get the corresponding echo canceller state */
        public static int SPEEX_PREPROCESS_GET_ECHO_STATE = 25;

        /** Set maximal gain increase in dB/second (int32) */
        public static int SPEEX_PREPROCESS_SET_AGC_INCREMENT = 26;

        /** Get maximal gain increase in dB/second (int32) */
        public static int SPEEX_PREPROCESS_GET_AGC_INCREMENT = 27;

        /** Set maximal gain decrease in dB/second (int32) */
        public static int SPEEX_PREPROCESS_SET_AGC_DECREMENT = 28;

        /** Get maximal gain decrease in dB/second (int32) */
        public static int SPEEX_PREPROCESS_GET_AGC_DECREMENT = 29;

        /** Set maximal gain in dB (int32) */
        public static int SPEEX_PREPROCESS_SET_AGC_MAX_GAIN = 30;

        /** Get maximal gain in dB (int32) */
        public static int SPEEX_PREPROCESS_GET_AGC_MAX_GAIN = 31;

        /*  Can't set loudness */
        /** Get loudness */
        public static int SPEEX_PREPROCESS_GET_AGC_LOUDNESS = 33;

        /*  Can't set gain */
        /** Get current gain (int32 percent) */
        public static int SPEEX_PREPROCESS_GET_AGC_GAIN = 35;

        /*  Can't set spectrum size */
        /** Get spectrum size for power spectrum (int32) */
        public static int SPEEX_PREPROCESS_GET_PSD_SIZE = 37;

        /*  Can't set power spectrum */
        /** Get power spectrum (int32[] of squared values) */
        public static int SPEEX_PREPROCESS_GET_PSD = 39;

        /*  Can't set noise size */
        /** Get spectrum size for noise estimate (int32)  */
        public static int SPEEX_PREPROCESS_GET_NOISE_PSD_SIZE = 41;

        /*  Can't set noise estimate */
        /** Get noise estimate (int32[] of squared values) */
        public static int SPEEX_PREPROCESS_GET_NOISE_PSD = 43;

        /* Can't set speech probability */
        /** Get speech probability in last frame (int32).  */
        public static int SPEEX_PREPROCESS_GET_PROB = 45;

        /** Set preprocessor Automatic Gain Control level (int32) */
        public static int SPEEX_PREPROCESS_SET_AGC_TARGET = 46;
        /** Get preprocessor Automatic Gain Control level (int32) */
        public static int SPEEX_PREPROCESS_GET_AGC_TARGET = 47;
    }
}
