using System;
using System.Threading;
using System.Threading.Tasks;

namespace AndroidMic.Audio
{
    // implementation of circular buffer
    public class AudioBuffer
    {
        public const int Capacity = 5 * 1024;
        public readonly byte[] Buffer = new byte[Capacity];

        private int regionSize = 0;
        private int regionLeft = 0;
        private int regionRight = 0;

        // can be accessed from at most one thread at a time
        private readonly SemaphoreSlim semaphore = new SemaphoreSlim(1, 1);

        ~AudioBuffer()
        {
            semaphore.Dispose();
        }

        /// <summary>
        /// Open buffer region to read from
        /// </summary>
        /// <param name="requestSize"></param>
        /// <param name="actualSize"></param>
        /// <param name="offset"></param>
        public async Task<Tuple<int, int>> OpenReadRegion(int requestSize)
        {
            await semaphore.WaitAsync();
            int actualSize = Math.Min(Math.Min(requestSize, regionSize), Capacity - regionLeft);
            int offset = regionLeft;
            return new Tuple<int, int>(actualSize, offset);
        }

        /// <summary>
        /// Close buffer read region
        /// </summary>
        /// <param name="readSize"></param>
        public void CloseReadRegion(int readSize)
        {
            regionLeft = (regionLeft + readSize) % Capacity;
            regionSize -= Math.Min(readSize, regionSize);
            semaphore.Release();
        }

        /// <summary>
        /// Open buffer region to write to
        /// </summary>
        /// <param name="requestSize"></param>
        /// <param name="actualSize"></param>
        /// <param name="offset"></param>
        public async Task<Tuple<int, int>> OpenWriteRegion(int requestSize)
        {
            await semaphore.WaitAsync();
            int actualSize = Math.Min(Math.Min(requestSize, Capacity - regionSize), Capacity - regionRight);
            int offset = regionRight;
            return new Tuple<int, int>(actualSize, offset);
        }

        /// <summary>
        /// Close buffer write region
        /// </summary>
        /// <param name="writeSize"></param>
        public void CloseWriteRegion(int writeSize)
        {
            regionRight = (regionRight + writeSize) % Capacity;
            regionSize = Math.Min(regionSize + writeSize, Capacity);
            semaphore.Release();
        }

        /// <summary>
        /// Get buffer size
        /// </summary>
        /// <returns></returns>
        public int Size()
        {
            semaphore.Wait();
            var result = regionSize;
            semaphore.Release();
            return result;
        }

        /// <summary>
        /// Whether buffer is empty
        /// </summary>
        /// <returns></returns>
        public bool IsEmpty()
        {
            semaphore.Wait();
            var result = regionSize == 0;
            semaphore.Release();
            return result;
        }

        /// <summary>
        /// Clear buffer
        /// </summary>
        public void Clear()
        {
            semaphore.Wait();
            regionLeft = 0;
            regionRight = 0;
            regionSize = 0;
            semaphore.Release();
        }
    }
}
