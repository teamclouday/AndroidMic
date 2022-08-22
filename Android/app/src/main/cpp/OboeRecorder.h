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

#define AUDIO_BUFFER_SIZE   2 * 1024

class OboeRecorder {
public:
    OboeRecorder();

    /// Get singleton
    static std::shared_ptr<OboeRecorder> Instance();

    /// Start recording
    void StartRecord();

    /// Stop recording
    void StopRecord();

    /// Set device ID
    void SetDeviceId(int32_t deviceId);

    /// Set sample rate
    void SetSampleRate(int32_t sampleRate);

    /// Set number of shorts (frames) in each buffer
    void SetBufferSizeInFrames(int32_t frames);

    /// Get audio buffer to read from
    std::shared_ptr<AudioBuffer<>> GetAudioBuffer();

    /// Check if is little endian
    bool IsLittleEndian() const;

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
    std::shared_ptr<AudioBuffer<>> _buffer;
    std::thread _readThread;

    volatile bool _recording;

    bool _isLittleEndian;
};

#endif //MICROPHONE_OBOERECORDER_H
