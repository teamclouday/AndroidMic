#ifndef MICROPHONE_AUDIOBUFFER_H
#define MICROPHONE_AUDIOBUFFER_H

#include <android/log.h>
#include <cstdint>
#include <array>
#include <vector>
#include <mutex>

template<uint32_t COUNT, typename TYPE = int16_t>
class AudioBuffer {
public:
    AudioBuffer(uint32_t bufferSize) {
        _bufferSize = bufferSize;
        for (auto &buffer: _buffers)
            buffer.resize(_bufferSize);
        Clear();
    }

    /// Get read buffer memory
    const TYPE *GetReadBuffer() {
        return _buffers[_bufferToRead].data();
    }

    /// Get read buffer current recorded size
    int32_t &GetReadBufferSize() {
        return _bufferValidCounts[_bufferToRead];
    }

    /// Get write buffer memory
    TYPE *GetWriteBuffer() {
        return _buffers[_bufferToWrite].data();
    }

    /// Get write buffer current recorded size
    int32_t &GetWriteBufferSize() {
        return _bufferValidCounts[_bufferToWrite];
    }

    /// Advance to next available read buffer
    void NextReadBuffer() {
        std::lock_guard<std::mutex> guard(_mutex);
        _bufferValidCounts[_bufferToRead] = 0;
        _bufferToRead++;
        validateBufferPointers();
    }

    /// Advance to next available write buffer
    void NextWriteBuffer() {
        std::lock_guard<std::mutex> guard(_mutex);
        _bufferToWrite++;
        validateBufferPointers();
    }

    /// Clear & reset all buffers
    void Clear() {
        for (auto &val: _bufferValidCounts)
            val = 0;
        _bufferToWrite = 0;
        _bufferToRead = 0;
        validateBufferPointers();
    }

private:
    /// Validate & fix current buffer locations
    void validateBufferPointers() {
        if (_bufferToWrite < 0) _bufferToWrite = COUNT - 1;
        else if (_bufferToWrite >= COUNT) _bufferToWrite = 0;

        if (_bufferToRead < 0) _bufferToRead = COUNT - 1;
        else if (_bufferToRead >= COUNT) _bufferToRead = 0;

        if (_bufferToRead == _bufferToWrite)
            _bufferToRead++;

        if (_bufferToRead < 0) _bufferToRead = COUNT - 1;
        else if (_bufferToRead >= COUNT) _bufferToRead = 0;

//        __android_log_print(ANDROID_LOG_DEBUG, "AudioBuffer", "[validateBufferPointers] R=%d W=%d",
//                            _bufferToRead, _bufferToWrite);
    }

private:
    std::array<std::vector<TYPE>, COUNT> _buffers;
    std::array<int32_t, COUNT> _bufferValidCounts;
    volatile uint32_t _bufferToWrite;
    volatile uint32_t _bufferToRead;
    uint32_t _bufferSize;

    std::mutex _mutex;
};

#endif //MICROPHONE_AUDIOBUFFER_H
