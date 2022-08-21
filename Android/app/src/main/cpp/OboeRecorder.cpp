#include "OboeRecorder.h"
#include <android/log.h>
#include <cstring>
#include <chrono>

#define LOGTAG "OboeRecorder"

OboeRecorder::OboeRecorder() : _deviceId(oboe::kUnspecified),
                               _sampleRate(16000), _bufferSizeInFrames(0),
                               _stream(nullptr), _buffers(nullptr), _recording(false) {
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
            ->setFormat(oboe::AudioFormat::I16)
            ->setChannelCount(oboe::Mono)
            ->setSampleRate(_sampleRate)
            ->setDeviceId(_deviceId)
            ->setDirection(oboe::Direction::Input)
            ->setContentType(oboe::Speech)
            ->setInputPreset(oboe::Generic)
            ->openStream(_stream);
    if (result != oboe::Result::OK)
        __android_log_print(ANDROID_LOG_DEBUG, LOGTAG, "[StartRecord] failed to open input stream");

    _stream->requestStart();

    if (_bufferSizeInFrames > _stream->getBufferCapacityInFrames())
        _bufferSizeInFrames = _stream->getBufferCapacityInFrames();

    if (_bufferSizeInFrames > 0)
        _stream->setBufferSizeInFrames(_bufferSizeInFrames);

    if (_stream->getBufferSizeInFrames() != _bufferSizeInFrames)
        __android_log_print(ANDROID_LOG_DEBUG, LOGTAG,
                            "[StartRecord] num frames changed from %d to %d", _bufferSizeInFrames,
                            _stream->getBufferSizeInFrames());

    _bufferSizeInFrames = _stream->getBufferSizeInFrames();

    _buffers = std::make_unique<AudioBuffer<AUDIO_BUFFER_COUNT>>(_bufferSizeInFrames);
    _copyBuffer.resize(_bufferSizeInFrames * 2);

    // skip first few frames
    int frames;
    do {
        auto res = _stream->read(_buffers->GetWriteBuffer(), _bufferSizeInFrames, 0);
        if (res != oboe::Result::OK) break;
        frames = res.value();
    } while (frames != 0);

    _recording = true;
    _readThread = std::thread(&OboeRecorder::readStream, this);

    __android_log_print(ANDROID_LOG_DEBUG, LOGTAG,
                        "[StartRecord] init %d buffers with size %d each",
                        AUDIO_BUFFER_COUNT, _bufferSizeInFrames);
}

void OboeRecorder::StopRecord() {
    if (_stream) {
        _recording = false;
        _readThread.join();
        _stream->stop();
        _stream->close();
        _stream = nullptr;
        _buffers = nullptr;
        _copyBuffer.clear();
        _copyBuffer.resize(0);
        __android_log_print(ANDROID_LOG_DEBUG, LOGTAG, "[StopRecord] fully stopped");
    }
}

bool OboeRecorder::HasBuffer() {
    return _buffers != nullptr;
}

std::mutex &OboeRecorder::GetLock() {
    return _mutex;
}

const int16_t *OboeRecorder::GetBuffer() const {
    return _buffers->GetReadBuffer();
}

const int8_t *OboeRecorder::GetByteBuffer() {
    // if big endian, reverse order
    // assuming Windows PC (NAudio) is little endian
    if (!_isLittleEndian) {
        auto ptr = _buffers->GetReadBuffer();
        for (int i = 0; i < _buffers->GetReadBufferSize(); ++i, ++ptr) {
            int16_t val = *ptr;
            _copyBuffer[i * 2] = (int8_t) ((val >> 8) & 0xff);
            _copyBuffer[i * 2 + 1] = (int8_t) (val & 0xff);
        }
    } else
        memcpy(_copyBuffer.data(), _buffers->GetReadBuffer(), _buffers->GetReadBufferSize() * 2);
    return _copyBuffer.data();
}

int32_t OboeRecorder::GetRecordedFrames() {
    auto res = _buffers->GetReadBufferSize();
    return res;
}

void OboeRecorder::ReleaseBuffer() {
    _buffers->NextReadBuffer();
}

void OboeRecorder::SetDeviceId(int32_t deviceId) {
    if (deviceId != _deviceId) {
        _deviceId = deviceId;
        restartStream();
    }
}

void OboeRecorder::SetSampleRate(int32_t sampleRate) {
    if (sampleRate != _sampleRate) {
        _sampleRate = sampleRate;
        restartStream();
    }
}

void OboeRecorder::SetBufferSizeInFrames(int32_t frames) {
    if (frames != _bufferSizeInFrames) {
        _bufferSizeInFrames = frames;
        restartStream();
    }
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
            _buffers->GetWriteBufferSize() = 0;
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
            continue;
        }
        _mutex.lock();
        auto &size = _buffers->GetWriteBufferSize();
        auto res = _stream->read(
                _buffers->GetWriteBuffer() + size,
                _bufferSizeInFrames - size, 0);
        if (res == oboe::Result::OK) {
            size += res.value();
            if (size >= _bufferSizeInFrames) {
                size = _bufferSizeInFrames;
                _buffers->NextWriteBuffer();
                _buffers->GetWriteBufferSize() = 0;
                __android_log_print(ANDROID_LOG_DEBUG, LOGTAG,
                                    "[readStream] current stream buffer full, swapped to new one");
            }
        }
        _mutex.unlock();
    }
}
