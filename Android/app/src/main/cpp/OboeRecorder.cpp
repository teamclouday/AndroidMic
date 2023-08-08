#include "OboeRecorder.h"
#include "Logging.h"

#include <cstring>
#include <chrono>
#include <cmath>

OboeRecorder::OboeRecorder() : _deviceId(oboe::kUnspecified),
                               _sampleRate(16000), _channelCount(oboe::Mono),
        // not sure if we can get rid of this cast
                               _audioFormat(static_cast<int32_t>(oboe::AudioFormat::I16)),
                               _bufferSizeInFrames(0),
                               _stream(nullptr), _buffer(nullptr), _recording(false) {
    // check system endian
    // reference: https://stackoverflow.com/questions/35683931/how-to-convert-byte-array-to-integral-types-int-long-short-etc-endian-saf
    short a = 1;
    _isLittleEndian = *((char *) &a) & 1;
}

std::shared_ptr<OboeRecorder> OboeRecorder::Instance() {
    static auto recorder = std::make_shared<OboeRecorder>();
    return recorder;
}

void OboeRecorder::StartRecord() {
    if (_stream) StopRecord();
    oboe::AudioStreamBuilder builder;
    auto result = builder.setSharingMode(oboe::SharingMode::Exclusive)
            ->setPerformanceMode(oboe::PerformanceMode::LowLatency)
            ->setFormat(static_cast<oboe::AudioFormat>(_audioFormat))
            ->setChannelCount(_channelCount)
            ->setSampleRate(_sampleRate)
            ->setDeviceId(_deviceId)
            ->setDirection(oboe::Direction::Input)
            ->setContentType(oboe::Speech)
            ->setInputPreset(oboe::Generic)
            ->openStream(_stream);
    if (result != oboe::Result::OK) {
        LOGD("[StartRecord] failed to open input stream");
    }

    _stream->requestStart();

    if (_bufferSizeInFrames > _stream->getBufferCapacityInFrames())
        _bufferSizeInFrames = _stream->getBufferCapacityInFrames();

    if (_bufferSizeInFrames > 0)
        _stream->setBufferSizeInFrames(_bufferSizeInFrames);

    if (_stream->getBufferSizeInFrames() != _bufferSizeInFrames) {
        LOGD("[StartRecord] num frames changed from %d to %d", _bufferSizeInFrames,
             _stream->getBufferSizeInFrames());
    }

    _bufferSizeInFrames = _stream->getBufferSizeInFrames();

    _buffer = std::make_unique<AudioBuffer<>>(std::max(AUDIO_BUFFER_SIZE, _bufferSizeInFrames));

    // skip first few frames
    int frames;
    do {
        uint32_t memSize;
        auto ptr = _buffer->OpenWriteMemoryRegion(_bufferSizeInFrames, memSize);
        auto res = _stream->read(ptr, (int32_t) memSize, 0);
        _buffer->CloseWriteMemoryRegion(memSize);
        _buffer->Clear();
        if (res != oboe::Result::OK) break;
        frames = res.value();
    } while (frames != 0);

    _recording = true;
    _readThread = std::thread(&OboeRecorder::readStream, this);

    LOGD("[StartRecord] init audio buffer with size %u", _buffer->Capacity());
}

void OboeRecorder::StopRecord() {
    if (_stream) {
        _recording = false;
        _readThread.join();
        _stream->stop();
        _stream->close();
        _stream = nullptr;
        _buffer = nullptr;
        LOGD("[StopRecord] fully stopped");
    }
}

void OboeRecorder::SetDeviceId(int32_t deviceId) {
    if (deviceId != _deviceId) {
        _deviceId = deviceId;
        restartStream();
        LOGD("[SetDeviceId] set device ID %d", _deviceId);
    }
}

void OboeRecorder::SetSampleRate(int32_t sampleRate) {
    if (sampleRate != _sampleRate) {
        _sampleRate = sampleRate;
        restartStream();
        LOGD("[SetDeviceId] set sample rate %d", _sampleRate);
    }
}

void OboeRecorder::SetChannelCount(int32_t channelCount) {
    if (channelCount != _channelCount) {
        _channelCount = channelCount;
        restartStream();
        LOGD("[SetDeviceId] set channel count %d", _channelCount);
    }
}

void OboeRecorder::SetAudioFormat(int32_t audioFormat) {
    if (audioFormat != _audioFormat) {
        _audioFormat = audioFormat;
        restartStream();
        LOGD("[SetDeviceId] set audio format %d", _audioFormat);
    }
}

void OboeRecorder::SetBufferSizeInFrames(int32_t frames) {
    if (frames != _bufferSizeInFrames) {
        _bufferSizeInFrames = frames;
        restartStream();
        LOGD("[SetDeviceId] set buffer size %d", _bufferSizeInFrames);
    }
}

std::shared_ptr<AudioBuffer<>> OboeRecorder::GetAudioBuffer() {
    return _buffer;
}

bool OboeRecorder::IsLittleEndian() const {
    return _isLittleEndian;
}

void OboeRecorder::restartStream() {
    if (_stream) {
        StopRecord();
        StartRecord();
    }
}

void OboeRecorder::readStream() {
    while (_recording) {
        if (!_stream) {
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
            continue;
        }
        uint32_t size;
        auto ptr = _buffer->OpenWriteMemoryRegion(_bufferSizeInFrames, size);
        auto res = _stream->read(ptr, (int32_t) size, 0);
        // if result is not OK, regard as if nothing is written
        if (res != oboe::Result::OK) {
            size = 0;
        } else {
            size = std::min(size, (uint32_t) res.value());
        }
        _buffer->CloseWriteMemoryRegion(size);
    }
}
