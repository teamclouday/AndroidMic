#ifndef MICROPHONE_AUDIOBUFFER_H
#define MICROPHONE_AUDIOBUFFER_H

#include "Logging.h"
#include <cstdint>
#include <array>
#include <vector>
#include <mutex>
#include <cmath>

// an implementation of circular buffer
template<typename TYPE = int16_t>
class AudioBuffer {
public:
    AudioBuffer(uint32_t capacity) {
        _bufferCapacity = capacity;
        _buffer.resize(capacity);
        _regionLeft = _regionRight = 0;
        _bufferSize = 0;
    }

    /// Open memory region to read from
    const TYPE *OpenReadMemoryRegion(uint32_t requestSize, uint32_t &regionSize) {
        _mutex.lock();
        regionSize = std::min(std::min(requestSize, _bufferSize),
                              _bufferCapacity - _regionLeft);
        return &_buffer[_regionLeft];
    }

    /// Close memory read region after open
    void CloseReadMemoryRegion(uint32_t readSize) {
        _regionLeft = (_regionLeft + readSize) % _bufferCapacity;
        _bufferSize -= std::min(readSize, _bufferSize);
        _mutex.unlock();
        if (readSize)
            LOGD("[AudioBuffer] read %u values", readSize);
    }

    /// Open memory region to write from
    TYPE *OpenWriteMemoryRegion(uint32_t requestSize, uint32_t &regionSize) {
        _mutex.lock();
        regionSize = std::min(std::min(requestSize, _bufferCapacity - _bufferSize),
                              _bufferCapacity - _regionRight);
        return &_buffer[_regionRight];
    }

    /// Close memory write region after open
    void CloseWriteMemoryRegion(uint32_t writeSize) {
        _regionRight = (_regionRight + writeSize) % _bufferCapacity;
        _bufferSize = std::min(_bufferSize + writeSize, _bufferCapacity);
        _mutex.unlock();
        if (writeSize)
            LOGD("[AudioBuffer] write %u values", writeSize);
    }

    /// Reset buffer
    void Clear() {
        std::lock_guard<std::mutex> lock(_mutex);
        _regionLeft = _regionRight = 0;
        _bufferSize = 0;
        LOGD("[AudioBuffer] cleared");
    }

    /// Get buffer current size
    uint32_t Size() {
        std::lock_guard<std::mutex> lock(_mutex);
        return _bufferSize;
    }

    /// Get buffer max capacity
    uint32_t Capacity() {
        std::lock_guard<std::mutex> lock(_mutex);
        return _bufferCapacity;
    }

    /// Check if buffer is empty
    bool IsEmpty() {
        std::lock_guard<std::mutex> lock(_mutex);
        return _bufferSize == 0;
    }

private:
    std::vector<TYPE> _buffer;
    uint32_t _bufferCapacity;
    uint32_t _bufferSize;
    uint32_t _regionLeft, _regionRight;
    std::mutex _mutex;
};

#endif //MICROPHONE_AUDIOBUFFER_H
