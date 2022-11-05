using System;
using System.Runtime.InteropServices;

namespace librnnoise
{
    using DenoiseState = IntPtr;
    using RNNModel = IntPtr;
    using FilePtr = IntPtr;

    // C# binding of librnnoise rnnoise.h header
    public sealed class Rnnoise
    {

        // define library path
        private const string dllpath = "librnnoise.dll";

        /**
         * Initializes a pre-allocated DenoiseState
         *
         * If model is NULL the default model is used.
         *
         * See: rnnoise_create() and rnnoise_model_from_file()
         */
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern int rnnoise_init(DenoiseState st, RNNModel model);

        public static int rnnoise_init(DenoiseState st)
        {
            // use default model
            return rnnoise_init(st, IntPtr.Zero);
        }

        /**
         * Allocate and initialize a DenoiseState
         *
         * If model is NULL the default model is used.
         *
         * The returned pointer MUST be freed with rnnoise_destroy().
         */
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern DenoiseState rnnoise_create(RNNModel model);

        public static DenoiseState rnnoise_create()
        {
            // use default model
            return rnnoise_create(IntPtr.Zero);
        }

        /**
         * Free a DenoiseState produced by rnnoise_create.
         *
         * The optional custom model must be freed by rnnoise_model_free() after.
         */
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern void rnnoise_destroy(DenoiseState st);

        /**
         * Denoise a frame of samples
         *
         * in and out must be at least rnnoise_get_frame_size() large.
         */
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern float rnnoise_process_frame(DenoiseState st, float[] outStream, float[] inStream);

        // byte array version, as long as they were copied from float array
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern float rnnoise_process_frame(DenoiseState st, byte[] outStream, byte[] inStream);

        /**
         * Load a model from a file
         *
         * It must be deallocated with rnnoise_model_free()
         */
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern RNNModel rnnoise_model_from_file(FilePtr f);

        /**
         * Free a custom model
         *
         * It must be called after all the DenoiseStates referring to it are freed.
         */
        [DllImport(dllpath, CallingConvention = CallingConvention.Cdecl)]
        public static extern void rnnoise_model_free(RNNModel model);
    }
}
