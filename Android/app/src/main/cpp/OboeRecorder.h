#ifndef MICROPHONE_OBOERECORDER_H
#define MICROPHONE_OBOERECORDER_H

#include <oboe/Oboe.h>
#include <memory>
#include <vector>
#include <thread>
#include <mutex>
#include <array>
#include <cstdint>

#include "AudioBuffer.h"

#define AUDIO_BUFFER_COUNT  3

class OboeRecorder {
public:
    OboeRecorder();

    /// Get singleton
    static std::shared_ptr<OboeRecorder> Instance();

    /// Start recording
    void StartRecord();

    /// Stop recording
    void StopRecord();

    /// Check if has recording buffer
    bool HasBuffer();

    /// Get mutex used by recording process
    std::mutex &GetLock();

    /// Get short array buffer
    const int16_t *GetBuffer() const;

    /// Get byte array buffer
    const int8_t *GetByteBuffer();

    /// Get number of shorts (frames) in buffer
    int32_t GetRecordedFrames();

    /// Release current read buffer (clear it and advance to next)
    void ReleaseBuffer();

    /// Set device ID
    void SetDeviceId(int32_t deviceId);

    /// Set sample rate
    void SetSampleRate(int32_t sampleRate);

    /// Set number of shorts (frames) in each buffer
    void SetBufferSizeInFrames(int32_t frames);

private:
    /// Restart recording
    void restartStream();

    /// Recording process in a separate thread
    void readStream();

private:
    int32_t _deviceId;
    int32_t _sampleRate;
    int32_t _bufferSizeInFrames;

    std::shared_ptr<oboe::AudioStream> _stream;
    std::unique_ptr<AudioBuffer<AUDIO_BUFFER_COUNT>> _buffers;
    std::vector<int8_t> _copyBuffer;
    std::mutex _mutex;
    std::thread _readThread;

    volatile bool _recording;

    bool _isLittleEndian;
};

#endif //MICROPHONE_OBOERECORDER_H
